// Page state for one replay: the loaded run, whether it plays, the pacing anchor, and the outcome.
// One requestAnimationFrame loop steps the simulator, voices its events, and redraws the view.

import { ArenaView, type PanelChrome } from './arena_view';
import { stepsDue } from './clock';
import type { Outcome } from './decode';
import { Replay } from './replay';
import { Soundtrack, voiceFor } from './sound';
import { tray } from './tray';
import { type Stamp, verdictFor } from './verdict';

const VOICES_PER_STEP = 6;

export class Player {
  private replay: Replay | null = null;
  private view: ArenaView | null = null;
  private sound: Soundtrack | null = null;
  private audio: AudioContext | null = null;
  private muted = false;
  private playing = false;
  private anchor_ms = 0;
  private done = 0;
  private outcome: Outcome | null = null;
  private stamp: Stamp | null = null;
  private stored: Outcome | null = null;
  private chrome: PanelChrome = { label: '', name: '', prediction: '' };
  private levelJson = '';
  private sceneJson = '';

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly dt_s: number,
    private readonly onStatus: (text: string) => void,
  ) {
    requestAnimationFrame(() => this.frame());
  }

  // Validates and starts a run at step 0, paused. Throws with the validation problems on refusal.
  load(levelJson: string, sceneJson: string, chrome: PanelChrome, stored: Outcome | null): void {
    this.replay?.free();
    this.replay = Replay.start(levelJson, sceneJson);
    this.levelJson = levelJson;
    this.sceneJson = sceneJson;
    this.chrome = chrome;
    this.stored = stored;
    const setup = this.replay.setup;
    this.view = new ArenaView(
      this.canvas,
      [{ chrome, setup, links: this.replay.links, items: tray(levelJson, sceneJson) }],
      setup.width_cm,
      setup.height_cm,
    );
    this.view.resize(this.canvas.parentElement?.clientWidth ?? this.view.grid.w);
    this.playing = false;
    this.done = 0;
    this.outcome = null;
    this.stamp = null;
    this.onStatus('ready');
  }

  play(): void {
    if (this.replay === null || this.outcome !== null) {
      return;
    }
    if (this.audio === null && !this.muted) {
      // Browsers only start audio from a user gesture, which is what called this.
      this.audio = new AudioContext();
      this.sound = new Soundtrack(this.audio);
    }
    this.anchor_ms = performance.now() - this.done * this.dt_s * 1000;
    this.playing = true;
    this.onStatus('playing');
  }

  pause(): void {
    this.playing = false;
    this.onStatus('paused');
  }

  toggle(): void {
    if (this.playing) {
      this.pause();
    } else {
      this.play();
    }
  }

  restart(): void {
    if (this.replay === null) {
      return;
    }
    const wasPlaying = this.playing || this.outcome !== null;
    this.load(this.levelJson, this.sceneJson, this.chrome, this.stored);
    if (wasPlaying) {
      this.play();
    }
  }

  setMuted(muted: boolean): void {
    this.muted = muted;
  }

  isPlaying(): boolean {
    return this.playing;
  }

  private frame(): void {
    requestAnimationFrame(() => this.frame());
    const replay = this.replay;
    const view = this.view;
    if (replay === null || view === null) {
      return;
    }
    if (this.playing && this.outcome === null) {
      const n = stepsDue((performance.now() - this.anchor_ms) / 1000, this.done, this.dt_s);
      for (let i = 0; i < n && this.outcome === null; i += 1) {
        this.advance(replay);
      }
    }
    const time_s = replay.stepIndex() * this.dt_s;
    view.draw([{ poses: replay.frame(), time_s, stamp: this.stamp }]);
  }

  private advance(replay: Replay): void {
    const events = replay.step();
    this.done += 1;
    this.voice(replay, events);
    this.outcome = replay.outcome();
    if (this.outcome !== null) {
      this.playing = false;
      const o = this.outcome;
      this.stamp = verdictFor(replay.setup.goal.kind, o, this.stored);
      const stored = this.stored === null ? '' : `, stored events ${this.stored.event_hash}, state ${this.stored.state_hash}`;
      this.onStatus(
        `${this.stamp.text} after ${o.steps} steps, chain ${o.chain_length}, ` +
          `events ${o.event_hash}, state ${o.state_hash}${stored}`,
      );
    }
  }

  private voice(replay: Replay, events: { kind: 'contact' | 'via' | 'goal_met' | 'at_rest' | 'left_world'; a: number; b: number }[]): void {
    const sound = this.sound;
    const audio = this.audio;
    if (sound === null || audio === null || this.muted) {
      return;
    }
    const lookOf = (part: number) => {
      const slot = replay.setup.slots.find((s) => s.part === part && s.visual.shape !== 'pivot');
      return slot && slot.visual.shape !== 'pivot' ? slot.visual.look : null;
    };
    let played = 0;
    for (const e of events) {
      const speed = e.kind === 'contact' ? Math.max(replay.speed(e.a), replay.speed(e.b)) : 0;
      const v = voiceFor(e, lookOf, speed, replay.setup.goal.kind);
      if (v !== null && played < VOICES_PER_STEP) {
        sound.play(v, audio.currentTime);
        played += 1;
      }
    }
  }
}
