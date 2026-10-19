// Page state for one or more replays on one clock: the loaded lanes, whether they play, the pacing
// anchor, and each lane's outcome. Paced, one requestAnimationFrame loop steps every running lane,
// voices its events, and redraws the panels that moved. Unpaced, `advance` does the stepping.

import { ArenaView, type PanelChrome } from './arena_view';
import { stepsDue } from './clock';
import { roomFor } from './fit';
import type { Outcome } from './decode';
import { gridEnded, plan } from './lanes';
import type { PanelFrame } from './panel_view';
import { Replay } from './replay';
import { Soundtrack } from './sound';
import { tray } from './tray';
import { stampText } from './stamp';
import { stampPlace } from './tuck';
import { type Stamp, verdictFor } from './verdict';
import { VOICES_PER_STEP, voicesAt } from './voices';

export interface LaneInput {
  chrome: PanelChrome;
  levelJson: string;
  sceneJson: string;
  stored: Outcome | null;
  // Set for a counted run that ended without a submission: the lane never steps and wears this.
  missing?: Stamp;
}

export type State = 'empty' | 'ready' | 'playing' | 'paused' | 'ended';


interface Lane {
  replay: Replay;
  stored: Outcome | null;
  outcome: Outcome | null;
  stamp: Stamp | null;
  // The grid clock, in steps, when the lane ended; its stamp tucks a hold later.
  endedAt: number | null;
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
  // Steps advanced since the load, running on past the last lane's end until its stamp has tucked.
  private clock = 0;

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
      stamp: input.missing ?? null,
      endedAt: input.missing ? 0 : null,
    }));
    this.clock = 0;
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

  // Sizes the canvas to its container width and to the height left on screen after everything
  // under it; the view picks the upscale. A phone fits the width alone, since its viewport height
  // moves with the address bar and the lines under the grid.
  fit(): void {
    if (!this.view) {
      return;
    }
    const room = roomFor(this.canvas);
    this.view.resize(room.w, room.h);
    // Sizing the backing store clears it, so every lane is painted again, ended ones included.
    this.draw(this.lanes.map(() => true));
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

  // Drops every lane and the view, so another drawer may take the canvas.
  unload(): void {
    for (const lane of this.lanes) {
      lane.replay.free();
    }
    this.inputs = [];
    this.lanes = [];
    this.view = null;
    this.playing = false;
    this.done = 0;
  }

  setMuted(muted: boolean): void {
    this.muted = muted;
  }

  isPlaying(): boolean {
    return this.playing;
  }

  // A lane has ended once it wears a stamp: its own verdict, or a miss stamped at the start.
  ended(): boolean {
    return gridEnded(this.lanes.map((l) => l.stamp !== null));
  }

  // Whether every lane has ended and every stamp has tucked into its header.
  settled(): boolean {
    return this.ended() && this.lanes.every((l) => this.tucked(l));
  }

  private tucked(lane: Lane): boolean {
    return lane.endedAt !== null && stampPlace((this.clock - lane.endedAt) * this.dt_s) === 'tucked';
  }

  // Each lane's stamp, null while it still runs.
  verdicts(): (Stamp | null)[] {
    return this.lanes.map((l) => l.stamp);
  }

  // Each lane's outcome, null while it still runs.
  outcomes(): (Outcome | null)[] {
    return this.lanes.map((l) => l.outcome);
  }

  // Steps every running lane `steps` times and redraws the lanes that moved or whose stamp tucked.
  // Returns whether the grid has ended.
  advance(steps: number): boolean {
    const wasSettled = this.settled();
    const before = this.lanes.map((l) => this.tucked(l));
    const counts = plan(steps, this.lanes.map((l) => l.stamp !== null));
    this.clock += steps;
    const moved = this.lanes.map((lane, i) => {
      const n = counts[i] ?? 0;
      for (let k = 0; k < n && lane.stamp === null; k += 1) {
        this.step(lane);
      }
      if (lane.stamp !== null && lane.endedAt === null) {
        lane.endedAt = this.clock;
      }
      return n > 0 || this.tucked(lane) !== before[i];
    });
    this.draw(moved);
    const ended = this.ended();
    if ((ended && this.playing) || (this.settled() && !wasSettled)) {
      this.playing = false;
      this.onChange(this);
    }
    return ended;
  }

  // Draws every lane once with its stored verdict stamped over the ones still running, for a
  // recording that stops early. Nothing is kept: the next step redraws the running lanes as they are.
  drawStoredVerdicts(): void {
    const frames: PanelFrame[] = this.lanes.map((lane) => {
      const stored = lane.stored;
      const stamp =
        lane.stamp ??
        (stored === null
          ? null
          : { text: stampText(lane.replay.setup.goal.kind, stored), tone: stored.goal_met ? 'success' : 'failure' });
      // A stored verdict lands big; a lane's own stamp keeps its place.
      const tucked = lane.stamp !== null && this.tucked(lane);
      return { poses: lane.replay.frame(), time_s: lane.replay.stepIndex() * this.dt_s, stamp, tucked };
    });
    this.view?.draw(frames);
  }

  // The clock runs while playing, and on after the end until the last stamp has tucked.
  private frame(): void {
    requestAnimationFrame(() => this.frame());
    const tucking = this.ended() && !this.settled();
    if (!this.playing && !tucking) {
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
        ? { poses: lane.replay.frame(), time_s: lane.replay.stepIndex() * this.dt_s, stamp: lane.stamp, tucked: this.tucked(lane) }
        : null,
    );
    this.view?.draw(frames);
  }
}

