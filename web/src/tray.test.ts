import { describe, expect, it } from 'vitest';
import { cellWidth, iconFor, KINDS, tray } from './tray';

const LEVEL = JSON.stringify({
  id: 'grand-tour',
  inventory: { beam: 6, domino: 8, seesaw: 1, ball: 2, funnel: 1, spring_pad: 1, crate: 2, pendulum: 1, bucket: 1, rope: 1 },
});
const SCENE = JSON.stringify({
  level: 'grand-tour',
  parts: [
    { id: 'b1', kind: 'beam' },
    { id: 'b2', kind: 'beam' },
    { id: 's1', kind: 'seesaw' },
    { id: 'k1', kind: 'ball', material: 'wood' },
  ],
});

describe('tray', () => {
  it('lists every kind the level allows, in the format order, with the scene\'s count used', () => {
    const items = tray(LEVEL, SCENE);
    expect(items.map((i) => i.kind)).toEqual([
      'beam', 'funnel', 'ball', 'domino', 'crate', 'seesaw', 'pendulum', 'spring_pad', 'bucket', 'rope',
    ]);
    expect(items[0]).toEqual({ kind: 'beam', allowed: 6, used: 2 });
    expect(items[2]).toEqual({ kind: 'ball', allowed: 2, used: 1 });
    expect(items[3]).toEqual({ kind: 'domino', allowed: 8, used: 0 });
  });

  it('is empty for a level with no inventory and ignores a scene it cannot read', () => {
    expect(tray(JSON.stringify({ id: 'x' }), SCENE)).toEqual([]);
    expect(tray(LEVEL, '{ not json')[0]).toEqual({ kind: 'beam', allowed: 6, used: 0 });
  });

  it('has an icon for every kind in the format', () => {
    expect(KINDS.length).toBe(12);
    for (const kind of KINDS) {
      const icon = iconFor(kind);
      expect(icon.shapes.length + icon.links.length, kind).toBeGreaterThan(0);
    }
  });
});

describe('cellWidth', () => {
  it('shares the width between the items up to a cap', () => {
    expect(cellWidth(10, 430)).toBe(43);
    expect(cellWidth(3, 430)).toBe(44);
    expect(cellWidth(0, 430)).toBe(44);
  });
});
