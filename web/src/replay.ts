// The typed face of the wasm Replay: JSON and typed arrays in, decoded setup, poses, events, and outcome out.

import init, { Replay as WasmReplay, dt_s, ranges as wasmRanges, rules_version } from '../pkg/clattertrap_wasm.js';
import { decodeEvents, decodeFrame, decodeOutcome, decodeSetup } from './decode';
import type { Outcome, Pose, ReplayEvent, Setup } from './decode';
import type { Ranges } from './edit_scene';
import { linksFrom, type Link } from './links';

export interface Wasm {
  dt_s: number;
  rules_version: number;
}

export async function loadWasm(): Promise<Wasm> {
  await init();
  return { dt_s: dt_s(), rules_version: rules_version() };
}

// The part size ranges of the rules, which the editor offers as its limits.
export function ranges(): Ranges {
  return JSON.parse(wasmRanges()) as Ranges;
}

// A scene built for drawing with its placement problems, which may be empty.
export interface Preview {
  replay: Replay;
  problems: string[];
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

  // Builds a scene even when parts overlap, sit outside the zones, or exceed the inventory, and
  // lists those problems. Throws, one problem per line, when a part cannot be built at all.
  static preview(levelJson: string, sceneJson: string): Preview {
    const inner = WasmReplay.preview(levelJson, sceneJson);
    const setup = decodeSetup(inner.setup());
    const problems = inner.problems().split('\n').filter((line) => line.length > 0);
    return { replay: new Replay(inner, setup, linksFrom(setup, sceneJson)), problems };
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
