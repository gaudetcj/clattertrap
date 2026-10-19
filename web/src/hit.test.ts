import { describe, expect, it } from 'vitest';
import type { Pose, Setup } from './decode';
import { boundsOf, partAt } from './hit';

const setup: Setup = {
  width_cm: 400,
  height_cm: 300,
  time_limit_s: 20,
  zones: [],
  goal: { kind: 'ring', target: 'bell', via: [] },
  names: ['ball', 'beam', 'bucket', 'pend', 'floor'],
  slots: [
    { part: 0, visual: { shape: 'circle', r_m: 0.03, look: 'steel' } },
    { part: 1, visual: { shape: 'box', w_m: 1, h_m: 0.02, look: 'wood' } },
    { part: 2, visual: { shape: 'trough', inner_w_m: 0.3, inner_h_m: 0.3, wall_m: 0.02, look: 'wood' } },
    { part: 3, visual: { shape: 'pivot' } },
  ],
};
const at = (x_m: number, y_m: number, angle_deg = 0): Pose => ({ x_m, y_m, angle_rad: (angle_deg * Math.PI) / 180, alive: true });
// The beam is turned 90 degrees, so it stands upright at x = 2.
const poses = [at(1, 1), at(2, 1, 90), at(3, 0), at(3.5, 2.5)];

describe('the part under a point', () => {
  it('finds a ball by its radius, a box in its own frame, and a trough by its walls', () => {
    expect(partAt(setup, poses, 1.02, 1.01)).toBe(0);
    expect(partAt(setup, poses, 1.1, 1)).toBeUndefined();
    expect(partAt(setup, poses, 2.005, 1.45)).toBe(1);
    expect(partAt(setup, poses, 2.3, 1)).toBeUndefined();
    expect(partAt(setup, poses, 3.16, 0.1)).toBe(2);
    expect(partAt(setup, poses, 3, 0.5)).toBeUndefined();
  });

  it('finds a pivot within a small reach and prefers the part drawn last', () => {
    expect(partAt(setup, poses, 3.52, 2.51)).toBe(3);
    const stacked = [at(1, 1), at(1, 1), at(3, 0), at(3.5, 2.5)];
    expect(partAt(setup, stacked, 1, 1)).toBe(1);
  });

  it('ignores bodies that are gone', () => {
    const gone = poses.map((p, i) => (i === 0 ? { ...p, alive: false } : p));
    expect(partAt(setup, gone, 1, 1)).toBeUndefined();
  });
});

describe('the bounds of a body', () => {
  it('follow the shape and its turn', () => {
    const box = { shape: 'box' as const, w_m: 1, h_m: 0.02, look: 'wood' as const };
    const flat = boundsOf(box, at(2, 1));
    expect(flat.min_x).toBeCloseTo(1.5, 6);
    expect(flat.max_x).toBeCloseTo(2.5, 6);
    expect(flat.min_y).toBeCloseTo(0.99, 6);
    const upright = boundsOf(box, at(2, 1, 90));
    expect(upright.min_x).toBeCloseTo(1.99, 6);
    expect(upright.max_y).toBeCloseTo(1.5, 6);
    expect(boundsOf({ shape: 'circle', r_m: 0.03, look: 'steel' }, at(1, 1))).toEqual({ min_x: 0.97, min_y: 0.97, max_x: 1.03, max_y: 1.03 });
    expect(boundsOf({ shape: 'trough', inner_w_m: 0.3, inner_h_m: 0.3, wall_m: 0.02, look: 'wood' }, at(3, 0))).toEqual({ min_x: 2.83, min_y: 0, max_x: 3.17, max_y: 0.32 });
    expect(boundsOf({ shape: 'pivot' }, at(3.5, 2.5))).toEqual({ min_x: 3.47, min_y: 2.47, max_x: 3.53, max_y: 2.53 });
  });
});
