// Lines the painter draws between bodies: a pendulum's rod, a bell's rope, and a rope part's two ends.
// None of these have a body of their own, so they are derived from the setup and the scene JSON.

import type { Setup } from './decode';

export interface Link {
  from: number;
  to: number;
}

interface RopePart {
  a: string;
  b: string;
}

// Reads only what a rope needs from the scene; the rest of the scene is the simulator's business.
function ropes(sceneJson: string): RopePart[] {
  const scene: unknown = JSON.parse(sceneJson);
  if (typeof scene !== 'object' || scene === null) {
    return [];
  }
  const parts = (scene as { parts?: unknown }).parts;
  if (!Array.isArray(parts)) {
    return [];
  }
  const found: RopePart[] = [];
  for (const p of parts as unknown[]) {
    if (typeof p !== 'object' || p === null) {
      continue;
    }
    const { kind, a, b } = p as { kind?: unknown; a?: unknown; b?: unknown };
    if (kind === 'rope' && typeof a === 'string' && typeof b === 'string') {
      found.push({ a, b });
    }
  }
  return found;
}

function firstBody(setup: Setup, part: number): number {
  return setup.slots.findIndex((s) => s.part === part && s.visual.shape !== 'pivot');
}

export function linksFrom(setup: Setup, sceneJson: string): Link[] {
  const links: Link[] = [];
  setup.slots.forEach((slot, i) => {
    if (slot.visual.shape === 'pivot') {
      const body = firstBody(setup, slot.part);
      if (body >= 0) {
        links.push({ from: i, to: body });
      }
    }
  });
  for (const rope of ropes(sceneJson)) {
    const a = setup.names.indexOf(rope.a);
    const b = setup.names.indexOf(rope.b);
    if (a < 0 || b < 0) {
      continue;
    }
    const from = firstBody(setup, a);
    const to = firstBody(setup, b);
    if (from >= 0 && to >= 0) {
      links.push({ from, to });
    }
  }
  return links;
}
