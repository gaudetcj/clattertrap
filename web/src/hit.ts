// Which body sits under a point, for picking parts in the editor: shapes are tested in their own
// frame with a little reach, and the body drawn last wins.

import type { Pose, Setup, Visual } from './decode';

// How far outside a shape a point may fall and still pick it, in metres.
const REACH_M = 0.015;
// A pivot has no size of its own; this is how close a point must come.
const PIVOT_REACH_M = 0.03;

function inside(visual: Visual, pose: Pose, x_m: number, y_m: number): boolean {
  const dx = x_m - pose.x_m;
  const dy = y_m - pose.y_m;
  switch (visual.shape) {
    case 'circle':
      return Math.hypot(dx, dy) <= visual.r_m + REACH_M;
    case 'pivot':
      return Math.hypot(dx, dy) <= PIVOT_REACH_M;
    case 'box': {
      const c = Math.cos(-pose.angle_rad);
      const s = Math.sin(-pose.angle_rad);
      const lx = dx * c - dy * s;
      const ly = dx * s + dy * c;
      return Math.abs(lx) <= visual.w_m / 2 + REACH_M && Math.abs(ly) <= visual.h_m / 2 + REACH_M;
    }
    case 'trough': {
      // The origin is the floor's underside centre; the walls rise from there.
      const half = visual.inner_w_m / 2 + visual.wall_m;
      const top = visual.inner_h_m + visual.wall_m;
      const onWall = Math.abs(Math.abs(dx) - (visual.inner_w_m / 2 + visual.wall_m / 2)) <= visual.wall_m / 2 + REACH_M;
      const onFloor = dy <= visual.wall_m + REACH_M;
      return Math.abs(dx) <= half + REACH_M && dy >= -REACH_M && dy <= top + REACH_M && (onWall || onFloor);
    }
  }
}

// The slot index of the body under the point, or undefined when none is.
export function partAt(setup: Setup, poses: Pose[], x_m: number, y_m: number): number | undefined {
  for (let i = setup.slots.length - 1; i >= 0; i -= 1) {
    const slot = setup.slots[i];
    const pose = poses[i];
    if (slot && pose && pose.alive && inside(slot.visual, pose, x_m, y_m)) {
      return i;
    }
  }
  return undefined;
}

export interface Bounds {
  min_x: number;
  min_y: number;
  max_x: number;
  max_y: number;
}

function round(v: number): number {
  return Math.round(v * 1e6) / 1e6;
}

// The axis-aligned box around a body in world metres, for an outline round a picked part.
export function boundsOf(visual: Visual, pose: Pose): Bounds {
  let hw: number;
  let hh: number;
  let cy = pose.y_m;
  switch (visual.shape) {
    case 'circle':
      hw = visual.r_m;
      hh = visual.r_m;
      break;
    case 'pivot':
      hw = PIVOT_REACH_M;
      hh = PIVOT_REACH_M;
      break;
    case 'box': {
      const c = Math.abs(Math.cos(pose.angle_rad));
      const s = Math.abs(Math.sin(pose.angle_rad));
      hw = (visual.w_m / 2) * c + (visual.h_m / 2) * s;
      hh = (visual.w_m / 2) * s + (visual.h_m / 2) * c;
      break;
    }
    case 'trough':
      hw = visual.inner_w_m / 2 + visual.wall_m;
      hh = (visual.inner_h_m + visual.wall_m) / 2;
      cy = pose.y_m + hh;
      break;
  }
  return { min_x: round(pose.x_m - hw), min_y: round(cy - hh), max_x: round(pose.x_m + hw), max_y: round(cy + hh) };
}
