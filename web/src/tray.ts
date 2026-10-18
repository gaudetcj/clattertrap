// The inventory tray: what the level let the model place, how much of it the scene used, and a
// small drawing of each kind. Shows the viewer what was possible against what was built.

import type { Pose, Visual } from './decode';

export interface TrayItem {
  kind: string;
  allowed: number;
  used: number;
}

export interface Icon {
  shapes: { visual: Visual; pose: Pose }[];
  links: [Pose, Pose][];
}

// Every part kind a level may offer, in the order docs/format.md lists them.
export const KINDS = [
  'beam', 'funnel', 'ball', 'domino', 'crate', 'seesaw',
  'pendulum', 'spring_pad', 'bucket', 'balloon', 'pin', 'rope',
] as const;

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

function parse(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

// How many parts of each kind the scene places; empty when the scene cannot be read.
function usedByKind(sceneJson: string): Map<string, number> {
  const used = new Map<string, number>();
  const scene = parse(sceneJson);
  const parts = isRecord(scene) ? scene['parts'] : undefined;
  if (!Array.isArray(parts)) {
    return used;
  }
  for (const part of parts) {
    if (isRecord(part) && typeof part['kind'] === 'string') {
      used.set(part['kind'], (used.get(part['kind']) ?? 0) + 1);
    }
  }
  return used;
}

// The level's inventory against the scene, one item per kind the level allows.
export function tray(levelJson: string, sceneJson: string): TrayItem[] {
  const level = parse(levelJson);
  const inventory = isRecord(level) ? level['inventory'] : undefined;
  if (!isRecord(inventory)) {
    return [];
  }
  const used = usedByKind(sceneJson);
  const items: TrayItem[] = [];
  for (const kind of KINDS) {
    const allowed = inventory[kind];
    if (typeof allowed === 'number' && allowed > 0) {
      items.push({ kind, allowed, used: used.get(kind) ?? 0 });
    }
  }
  return items;
}

function at(x_m: number, y_m: number, angle_deg = 0): Pose {
  return { x_m, y_m, angle_rad: (angle_deg * Math.PI) / 180, alive: true };
}

const STEEL_BALL: Visual = { shape: 'circle', r_m: 0.03, look: 'steel' };
const PIVOT: Visual = { shape: 'pivot' };

// A small true-scale drawing of each kind, in metres around the icon's centre.
export function iconFor(kind: string): Icon {
  switch (kind) {
    case 'beam':
      return { shapes: [{ visual: { shape: 'box', w_m: 0.14, h_m: 0.02, look: 'wood' }, pose: at(0, 0, -12) }], links: [] };
    case 'funnel':
      return {
        shapes: [
          { visual: { shape: 'box', w_m: 0.09, h_m: 0.015, look: 'wood' }, pose: at(-0.035, 0.01, -55) },
          { visual: { shape: 'box', w_m: 0.09, h_m: 0.015, look: 'wood' }, pose: at(0.035, 0.01, 55) },
        ],
        links: [],
      };
    case 'ball':
      return { shapes: [{ visual: STEEL_BALL, pose: at(0, 0) }], links: [] };
    case 'domino':
      return { shapes: [{ visual: { shape: 'box', w_m: 0.02, h_m: 0.08, look: 'domino' }, pose: at(0, 0) }], links: [] };
    case 'crate':
      return { shapes: [{ visual: { shape: 'box', w_m: 0.07, h_m: 0.07, look: 'wood' }, pose: at(0, 0) }], links: [] };
    case 'seesaw':
      return {
        shapes: [
          { visual: { shape: 'box', w_m: 0.14, h_m: 0.02, look: 'wood' }, pose: at(0, 0.01, -10) },
          { visual: PIVOT, pose: at(0, 0) },
        ],
        links: [],
      };
    case 'pendulum':
      return { shapes: [{ visual: PIVOT, pose: at(0, 0.05) }, { visual: STEEL_BALL, pose: at(0.03, -0.02) }], links: [[at(0, 0.05), at(0.03, -0.02)]] };
    case 'spring_pad':
      return { shapes: [{ visual: { shape: 'box', w_m: 0.08, h_m: 0.02, look: 'pad' }, pose: at(0, 0.02) }], links: [] };
    case 'bucket':
      return { shapes: [{ visual: { shape: 'trough', inner_w_m: 0.06, inner_h_m: 0.05, wall_m: 0.01, look: 'wood' }, pose: at(0, -0.035) }], links: [] };
    case 'balloon':
      return { shapes: [{ visual: { shape: 'circle', r_m: 0.035, look: 'balloon' }, pose: at(0, 0.01) }], links: [] };
    case 'pin':
      return { shapes: [{ visual: { shape: 'box', w_m: 0.08, h_m: 0.01, look: 'pin' }, pose: at(0, 0, -90) }], links: [] };
    case 'rope':
      return { shapes: [], links: [[at(-0.05, 0.03), at(0.05, -0.03)]] };
    default:
      return { shapes: [], links: [] };
  }
}

// The widest a tray cell gets: an icon, a gap, and a three-character count.
const CELL_MAX_PX = 44;

// How wide each tray cell is when `count` items share `w` pixels.
export function cellWidth(count: number, w: number): number {
  return count === 0 ? CELL_MAX_PX : Math.min(CELL_MAX_PX, Math.floor(w / count));
}
