import { describe, expect, it } from 'vitest';
import { gridEnded, plan } from './lanes';

describe('lockstep lanes', () => {
  it('give every running lane the steps due and an ended lane none', () => {
    expect(plan(3, [false, true, false])).toEqual([3, 0, 3]);
    expect(plan(0, [false, false])).toEqual([0, 0]);
    expect(plan(2, [])).toEqual([]);
  });

  it('end the grid only when every lane has ended', () => {
    expect(gridEnded([true, true])).toBe(true);
    expect(gridEnded([true, false])).toBe(false);
    expect(gridEnded([])).toBe(true);
  });
});
