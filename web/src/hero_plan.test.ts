import { describe, expect, it } from 'vitest';
import { gifTakes } from './hero_plan';

describe('the GIF window', () => {
  it('takes every frame up to the lead-in plus the window and none after', () => {
    expect(gifTakes(1, 1, 6, 30)).toBe(true);
    expect(gifTakes(210, 1, 6, 30)).toBe(true);
    expect(gifTakes(211, 1, 6, 30)).toBe(false);
  });
});
