// The typed face of the wasm Replay: JSON and typed arrays in, decoded setup, poses, events, and outcome out.

import init, { Replay as WasmReplay, dt_s, rules_version } from '../pkg/clattertrap_wasm.js';
import { decodeEvents, decodeFrame, decodeOutcome, decodeSetup } from './decode';
import type { Outcome, Pose, ReplayEvent, Setup } from './decode';
import { linksFrom, type Link } from './links';

export interface Wasm {
  dt_s: number;
  rules_version: number;
}

export async function loadWasm(): Promise<Wasm> {
  await init();
  return { dt_s: dt_s(), rules_version: rules_version() };
}

export class Replay {
  private constructor(
    private readonly inner: WasmReplay,
    readonly setup: Setup,
    readonly links: Link[],
  ) {}

  // Throws with the validation problems, one per line, when the scene is refused.
  static start(levelJson: string, sceneJson: string): Replay {
    const inner = new WasmReplay(levelJson, sceneJson);
    const setup = decodeSetup(inner.setup());
    return new Replay(inner, setup, linksFrom(setup, sceneJson));
  }

  step(): ReplayEvent[] {
    return decodeEvents(this.inner.step());
  }

  frame(): Pose[] {
    return decodeFrame(this.inner.frame());
  }

  stepIndex(): number {
    return this.inner.step_index();
  }

  speed(part: number): number {
    const v = this.inner.velocity(part);
    return v.length === 2 ? Math.hypot(v[0] ?? 0, v[1] ?? 0) : 0;
  }

  outcome(): Outcome | null {
    const json = this.inner.outcome();
    return json === undefined ? null : decodeOutcome(json);
  }

  free(): void {
    this.inner.free();
  }
}
