// Types and decoders for what the wasm Replay hands out: setup JSON, event triples, frames, and the outcome.
// Every decoder checks the shape it was given and throws on anything unexpected.

export const LOOKS = ['wood', 'steel', 'rubber', 'domino', 'balloon', 'pad', 'pin', 'brass', 'wicker', 'orange', 'blue'] as const;
export type Look = (typeof LOOKS)[number];

export type Visual =
  | { shape: 'circle'; r_m: number; look: Look }
  | { shape: 'box'; w_m: number; h_m: number; look: Look }
  | { shape: 'trough'; inner_w_m: number; inner_h_m: number; wall_m: number; look: Look }
  | { shape: 'pivot' };

export interface Zone {
  x_cm: number;
  y_cm: number;
  width_cm: number;
  height_cm: number;
}

export const GOAL_KINDS = ['ring', 'press', 'fill', 'pop'] as const;
export type GoalKind = (typeof GOAL_KINDS)[number];

export interface Goal {
  kind: GoalKind;
  target: string;
  via: string[];
  count?: number;
}

export interface Slot {
  part: number;
  visual: Visual;
}

export interface Setup {
  width_cm: number;
  height_cm: number;
  time_limit_s: number;
  zones: Zone[];
  goal: Goal;
  names: string[];
  slots: Slot[];
}

export const EVENT_KINDS = ['contact', 'via', 'goal_met', 'at_rest', 'left_world'] as const;
export type EventKind = (typeof EVENT_KINDS)[number];

export interface ReplayEvent {
  kind: EventKind;
  a: number;
  b: number;
}

export interface Pose {
  x_m: number;
  y_m: number;
  angle_rad: number;
  alive: boolean;
}

export const ENDS = ['goal_met', 'time_limit', 'all_asleep'] as const;
export type End = (typeof ENDS)[number];

export interface Outcome {
  goal_met: boolean;
  end: End;
  time_s: number;
  steps: number;
  parts_used: number;
  chain_length: number;
  event_hash: string;
  state_hash: string;
  rules_version: number;
}

type Json = Record<string, unknown>;

function object(value: unknown, where: string): Json {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new Error(`${where}: expected an object`);
  }
  return value as Json;
}

function number(o: Json, key: string, where: string): number {
  const v = o[key];
  if (typeof v !== 'number' || !Number.isFinite(v)) {
    throw new Error(`${where}.${key}: expected a number`);
  }
  return v;
}

function string(o: Json, key: string, where: string): string {
  const v = o[key];
  if (typeof v !== 'string') {
    throw new Error(`${where}.${key}: expected a string`);
  }
  return v;
}

function boolean(o: Json, key: string, where: string): boolean {
  const v = o[key];
  if (typeof v !== 'boolean') {
    throw new Error(`${where}.${key}: expected a boolean`);
  }
  return v;
}

function array(o: Json, key: string, where: string): unknown[] {
  const v = o[key];
  if (!Array.isArray(v)) {
    throw new Error(`${where}.${key}: expected an array`);
  }
  return v;
}

function oneOf<T extends string>(value: string, allowed: readonly T[], where: string): T {
  const hit = allowed.find((a) => a === value);
  if (hit === undefined) {
    throw new Error(`${where}: unknown value ${JSON.stringify(value)}`);
  }
  return hit;
}

function look(o: Json, where: string): Look {
  return oneOf(string(o, 'look', where), LOOKS, `${where}.look`);
}

function visual(value: unknown, where: string): Visual {
  const o = object(value, where);
  const shape = string(o, 'shape', where);
  switch (shape) {
    case 'circle':
      return { shape, r_m: number(o, 'r_m', where), look: look(o, where) };
    case 'box':
      return { shape, w_m: number(o, 'w_m', where), h_m: number(o, 'h_m', where), look: look(o, where) };
    case 'trough':
      return {
        shape,
        inner_w_m: number(o, 'inner_w_m', where),
        inner_h_m: number(o, 'inner_h_m', where),
        wall_m: number(o, 'wall_m', where),
        look: look(o, where),
      };
    case 'pivot':
      return { shape };
    default:
      throw new Error(`${where}.shape: unknown value ${JSON.stringify(shape)}`);
  }
}

function zone(value: unknown, where: string): Zone {
  const o = object(value, where);
  return {
    x_cm: number(o, 'x_cm', where),
    y_cm: number(o, 'y_cm', where),
    width_cm: number(o, 'width_cm', where),
    height_cm: number(o, 'height_cm', where),
  };
}

function goal(value: unknown, where: string): Goal {
  const o = object(value, where);
  const kind = oneOf(string(o, 'kind', where), GOAL_KINDS, `${where}.kind`);
  const via = array(o, 'via', where).map((v, i) => {
    if (typeof v !== 'string') {
      throw new Error(`${where}.via[${i}]: expected a string`);
    }
    return v;
  });
  const g: Goal = { kind, target: string(o, 'target', where), via };
  if (kind === 'fill') {
    g.count = number(o, 'count', where);
  }
  return g;
}

export function decodeSetup(json: string): Setup {
  const o = object(JSON.parse(json), 'setup');
  return {
    width_cm: number(o, 'width_cm', 'setup'),
    height_cm: number(o, 'height_cm', 'setup'),
    time_limit_s: number(o, 'time_limit_s', 'setup'),
    zones: array(o, 'zones', 'setup').map((z, i) => zone(z, `setup.zones[${i}]`)),
    goal: goal(o['goal'], 'setup.goal'),
    names: array(o, 'names', 'setup').map((n, i) => {
      if (typeof n !== 'string') {
        throw new Error(`setup.names[${i}]: expected a string`);
      }
      return n;
    }),
    slots: array(o, 'slots', 'setup').map((s, i) => {
      const where = `setup.slots[${i}]`;
      const so = object(s, where);
      return { part: number(so, 'part', where), visual: visual(so['visual'], `${where}.visual`) };
    }),
  };
}

export function decodeOutcome(json: string): Outcome {
  const o = object(JSON.parse(json), 'outcome');
  return {
    goal_met: boolean(o, 'goal_met', 'outcome'),
    end: oneOf(string(o, 'end', 'outcome'), ENDS, 'outcome.end'),
    time_s: number(o, 'time_s', 'outcome'),
    steps: number(o, 'steps', 'outcome'),
    parts_used: number(o, 'parts_used', 'outcome'),
    chain_length: number(o, 'chain_length', 'outcome'),
    event_hash: string(o, 'event_hash', 'outcome'),
    state_hash: string(o, 'state_hash', 'outcome'),
    rules_version: number(o, 'rules_version', 'outcome'),
  };
}

// Event kinds arrive as the index into EVENT_KINDS, three numbers per event.
export function decodeEvents(triples: Uint16Array): ReplayEvent[] {
  const events: ReplayEvent[] = [];
  for (let i = 0; i + 2 < triples.length; i += 3) {
    const kind = EVENT_KINDS[triples[i] ?? 0];
    if (kind === undefined) {
      throw new Error(`event kind ${triples[i]} is unknown`);
    }
    events.push({ kind, a: triples[i + 1] ?? 0, b: triples[i + 2] ?? 0 });
  }
  return events;
}

// Four numbers per slot: x, y, angle, and 1 while the body exists.
export function decodeFrame(frame: Float32Array): Pose[] {
  const poses: Pose[] = [];
  for (let i = 0; i + 3 < frame.length; i += 4) {
    poses.push({
      x_m: frame[i] ?? 0,
      y_m: frame[i + 1] ?? 0,
      angle_rad: frame[i + 2] ?? 0,
      alive: (frame[i + 3] ?? 0) > 0.5,
    });
  }
  return poses;
}
