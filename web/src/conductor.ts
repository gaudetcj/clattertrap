// Page state for one or more replays on one clock: the loaded lanes, whether they play, the pacing
// anchor, and each lane's outcome. Paced, one requestAnimationFrame loop steps every running lane,
// voices its events, and redraws the panels that moved. Unpaced, `advance` does the stepping.

import { ArenaView, type PanelChrome } from './arena_view';
import { stepsDue } from './clock';
import type { Outcome } from './decode';
import { gridEnded, plan } from './lanes';
import type { PanelFrame } from './panel_view';
import { Replay } from './replay';
import { Soundtrack } from './sound';
import { tray } from './tray';
import { type Stamp, verdictFor } from './verdict';
import { VOICES_PER_STEP, voicesAt } from './voices';

export interface LaneInput {
  chrome: PanelChrome;
  levelJson: string;
  sceneJson: string;
  stored: Outcome | null;
}

export type State = 'empty' | 'ready' | 'playing' | 'paused' | 'ended';

interface Lane {
  replay: Replay;
  stored: Outcome | null;
  outcome: Outcome | null;
  stamp: Stamp | null;
}

export class Conductor {
  private inputs: LaneInput[] = [];
  private lanes: Lane[] = [];
  private view: ArenaView | null = null;
  private sound: Soundtrack | null = null;
  private audio: AudioContext | null = null;
  private muted = false;
  private playing = false;
  private anchor_ms = 0;
  private done = 0;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly dt_s: number,
    paced: boolean,
    private readonly onChange: (c: Conductor) => void,
  ) {
    if (paced) {
      requestAnimationFrame(() => this.frame());
    }
  }

  // Validates and starts every lane at step 0, paused. Throws with the problems of the first lane
  // the rules refuse.
  load(inputs: LaneInput[]): void {
    for (const lane of this.lanes) {
      lane.replay.free();
    }
    this.inputs = inputs;
    this.lanes = inputs.map((input) => ({
      replay: Replay.start(input.levelJson, input.sceneJson),
      stored: input.stored,
      outcome: null,
      stamp: null,
    }));
    const first = this.lanes[0]?.replay.setup;
    this.view = new ArenaView(
      this.canvas,
      this.lanes.map((lane, i) => ({
        chrome: inputs[i]?.chrome ?? { label: '', name: '', prediction: '' },
        setup: lane.replay.setup,
        links: lane.replay.links,
        items: tray(inputs[i]?.levelJson ?? '', inputs[i]?.sceneJson ?? ''),
      })),
      first?.width_cm ?? 400,
      first?.height_cm ?? 300,
    );
    this.fit();
    this.playing = false;
    this.done = 0;
    this.sound?.setVoices(this.lanes.length);
    this.draw(this.lanes.map(() => true));
    this.onChange(this);
  }

  // Sizes the canvas to its container; the view picks the whole-number upscale.
  fit(): void {
    this.view?.resize(this.canvas.parentElement?.clientWidth ?? this.view.grid.w);
  }

  // The low-res size of the grid, for a recorder.
  size(): { w: number; h: number } {
    return this.view?.grid ?? { w: 0, h: 0 };
  }

  state(): State {
    if (this.lanes.length === 0) {
      return 'empty';
    }
    if (this.ended()) {
      return 'ended';
    }
    if (this.playing) {
      return 'playing';
    }
    return this.done === 0 ? 'ready' : 'paused';
  }

  play(): void {
    if (this.lanes.length === 0 || this.ended()) {
      return;
    }
    if (this.audio === null && !this.muted) {
      // Browsers only start audio from a user gesture, which is what called this.
      this.audio = new AudioContext();
      this.sound = new Soundtrack(this.audio);
      this.sound.setVoices(this.lanes.length);
    }
    this.anchor_ms = performance.now() - this.done * this.dt_s * 1000;
    this.playing = true;
    this.onChange(this);
  }

  pause(): void {
    this.playing = false;
    this.onChange(this);
  }

  toggle(): void {
    if (this.playing) {
      this.pause();
    } else {
      this.play();
    }
  }

  restart(): void {
    if (this.lanes.length === 0) {
      return;
    }
    const resume = this.playing || this.ended();
    this.load(this.inputs);
    if (resume) {
      this.play();
    }
  }

  setMuted(muted: boolean): void {
    this.muted = muted;
  }

  isPlaying(): boolean {
    return this.playing;
  }

  ended(): boolean {
    return gridEnded(this.lanes.map((l) => l.outcome !== null));
  }

  // Each lane's stamp, null while it still runs.
  verdicts(): (Stamp | null)[] {
    return this.lanes.map((l) => l.stamp);
  }

  // Each lane's outcome, null while it still runs.
  outcomes(): (Outcome | null)[] {
    return this.lanes.map((l) => l.outcome);
  }

  // Steps every running lane `steps` times and redraws the lanes that moved. Returns whether the
  // grid has ended.
  advance(steps: number): boolean {
    const counts = plan(steps, this.lanes.map((l) => l.outcome !== null));
    const moved = this.lanes.map((lane, i) => {
      const n = counts[i] ?? 0;
      for (let k = 0; k < n && lane.outcome === null; k += 1) {
        this.step(lane);
      }
      return n > 0;
    });
    this.draw(moved);
    const ended = this.ended();
    if (ended && this.playing) {
      this.playing = false;
      this.onChange(this);
    }
    return ended;
  }

  private frame(): void {
    requestAnimationFrame(() => this.frame());
    if (!this.playing || this.ended()) {
      return;
    }
    const n = stepsDue((performance.now() - this.anchor_ms) / 1000, this.done, this.dt_s);
    this.done += n;
    if (n > 0) {
      this.advance(n);
    }
  }

  private step(lane: Lane): void {
    const events = lane.replay.step();
    if (this.sound !== null && this.audio !== null && !this.muted) {
      for (const v of voicesAt(lane.replay, events, VOICES_PER_STEP)) {
        this.sound.play(v, this.audio.currentTime);
      }
    }
    lane.outcome = lane.replay.outcome();
    if (lane.outcome !== null) {
      lane.stamp = verdictFor(lane.replay.setup.goal.kind, lane.outcome, lane.stored);
      this.onChange(this);
    }
  }

  private draw(moved: boolean[]): void {
    const frames: (PanelFrame | null)[] = this.lanes.map((lane, i) =>
      moved[i]
        ? { poses: lane.replay.frame(), time_s: lane.replay.stepIndex() * this.dt_s, stamp: lane.stamp }
        : null,
    );
    this.view?.draw(frames);
  }
}
