import { describe, expect, it } from 'vitest';
import type { Setup } from './decode';
import { linksFrom } from './links';

const setup: Setup = {
  width_cm: 400,
  height_cm: 300,
  time_limit_s: 20,
  zones: [],
  goal: { kind: 'ring', target: 'bell', via: [] },
  names: ['k1', 'bell', 'p', 'k2', 'r', 'floor'],
  slots: [
    { part: 5, visual: { shape: 'box', w_m: 4.5, h_m: 1, look: 'wood' } },
    { part: 0, visual: { shape: 'circle', r_m: 0.06, look: 'steel' } },
    { part: 1, visual: { shape: 'circle', r_m: 0.1, look: 'brass' } },
    { part: 1, visual: { shape: 'pivot' } },
    { part: 2, visual: { shape: 'pivot' } },
    { part: 2, visual: { shape: 'circle', r_m: 0.05, look: 'steel' } },
    { part: 3, visual: { shape: 'circle', r_m: 0.06, look: 'wood' } },
  ],
};

const scene = JSON.stringify({
  level: 'x',
  parts: [
    { id: 'p', kind: 'pendulum', x_cm: 1, y_cm: 2, length_cm: 50 },
    { id: 'k2', kind: 'ball', x_cm: 1, y_cm: 2, material: 'wood' },
    { id: 'r', kind: 'rope', a: 'k1', b: 'k2', max_length_cm: 40 },
  ],
});

describe('linksFrom', () => {
  it('joins every pivot to the body of the same part, whichever comes first', () => {
    const links = linksFrom(setup, scene);
    expect(links).toContainEqual({ from: 3, to: 2 });
    expect(links).toContainEqual({ from: 4, to: 5 });
  });

  it('joins the two parts of a rope by their first bodies', () => {
    expect(linksFrom(setup, scene)).toContainEqual({ from: 1, to: 6 });
  });

  it('ignores a rope whose end is unknown', () => {
    const bad = scene.replace('"b":"k2"', '"b":"nope"');
    expect(linksFrom(setup, bad)).toHaveLength(2);
  });
});
