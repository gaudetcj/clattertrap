// Page state for one replay: the loaded run, whether it plays, the pacing anchor, and the outcome.
// One requestAnimationFrame loop steps the simulator, voices its events, and redraws the view.

import { stepsDue } from './clock';
import type { Outcome } from './decode';
import { LevelView } from './level_view';
import { Replay } from './replay';
import { Soundtrack, voiceFor } from './sound';
import { stampText } from './stamp';

const VOICES_PER_STEP = 6;

export class Player {
  private replay: Replay | null = null;
  private view: LevelView | null = null;
  private sound: Soundtrack | null = null;
  private audio: AudioContext | null = null;
  private muted = false;
  private playing = false;
  private anchor_ms = 0;
  private done = 0;
  private outcome: Outcome | null = null;
  private levelJson = '';
  private sceneJson = '';
  private cm_per_px: number | null = null;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly dt_s: number,
    private readonly onStatus: (text: string) => void,
  ) {
    requestAnimationFrame(() => this.frame());
  }

  // Validates and starts a run at step 0, paused. Throws with the validation problems on refusal.
  load(levelJson: string, sceneJson: string): void {
    this.replay?.free();
    this.replay = Replay.start(levelJson, sceneJson);
    this.levelJson = levelJson;
    this.sceneJson = sceneJson;
    this.view = new LevelView(this.canvas, this.replay.setup, this.replay.links, this.cm_per_px);
    this.fit();
    this.playing = false;
    this.done = 0;
    this.outcome = null;
    this.onStatus('ready');
  }

  // Switches between the paper look (null) and 16-bit pixels of the given size.
  setLook(cm_per_px: number | null): void {
    this.cm_per_px = cm_per_px;
    this.view?.setLook(cm_per_px);
    this.fit();
  }

  // Lets the canvas take its stylesheet size again before the view sizes its backing store.
  private fit(): void {
    this.canvas.style.width = '';
    this.canvas.style.height = '';
    this.view?.resize(this.canvas.clientWidth, this.canvas.clientHeight);
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
    this.load(this.levelJson, this.sceneJson);
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
    const stamp = this.outcome === null ? null : stampText(replay.setup.goal.kind, this.outcome);
    view.draw(replay.frame(), time_s, stamp, this.outcome?.goal_met ?? false);
  }

  private advance(replay: Replay): void {
    const events = replay.step();
    this.done += 1;
    this.voice(replay, events);
    this.outcome = replay.outcome();
    if (this.outcome !== null) {
      this.playing = false;
      const o = this.outcome;
      this.onStatus(
        `${stampText(replay.setup.goal.kind, o)} after ${o.steps} steps, chain ${o.chain_length}, ` +
          `events ${o.event_hash}, state ${o.state_hash}`,
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
