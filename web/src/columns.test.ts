import { describe, expect, it } from 'vitest';
import { bestColumns, fitScale, panelSize } from './layout';

describe('column count', () => {
  const panel = panelSize(400, 300);

  it('picks the columns that give the biggest grid on the screen, wider on ties', () => {
    // Six lanes on a laptop: three across fits at one pixel per centimetre.
    expect(bestColumns(6, 1248, 650, panel)).toBe(3);
    // Twelve lanes: four across on a desktop, six across on an ultrawide.
    expect(bestColumns(12, 1920, 1000, panel)).toBe(4);
    expect(bestColumns(12, 3440, 900, panel)).toBe(6);
    // Never more columns than lanes, and at least one.
    expect(bestColumns(2, 3440, 900, panel)).toBe(2);
    expect(bestColumns(1, 500, 500, panel)).toBe(1);
    expect(bestColumns(0, 500, 500, panel)).toBe(1);
  });
});

describe('a box with no height limit', () => {
  // A phone fits the grid to its width alone, so the panels stack in one column.
  it('takes one column and fills the width', () => {
    const panel = panelSize(400, 300);
    expect(bestColumns(6, 370, Number.POSITIVE_INFINITY, panel)).toBe(1);
    expect(fitScale(370, Number.POSITIVE_INFINITY, { w: 400, h: 2424 })).toBeCloseTo(370 / 400, 6);
    expect(fitScale(900, Number.POSITIVE_INFINITY, { w: 400, h: 2424 })).toBe(2);
  });
});
