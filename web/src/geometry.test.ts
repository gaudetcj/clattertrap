import { describe, expect, it } from 'vitest';
import { beamEnds } from './geometry';

describe('beamEnds', () => {
  it('puts a horizontal beam ends half a length either side of the centre', () => {
    const ends = beamEnds({ x_cm: 100, y_cm: 50 }, 80, 0);
    expect(ends.left.x_cm).toBeCloseTo(60);
    expect(ends.left.y_cm).toBeCloseTo(50);
    expect(ends.right.x_cm).toBeCloseTo(140);
    expect(ends.right.y_cm).toBeCloseTo(50);
  });

  it('raises the right end for a positive counterclockwise angle', () => {
    const ends = beamEnds({ x_cm: 0, y_cm: 0 }, 100, 30);
    expect(ends.right.x_cm).toBeCloseTo(43.301, 3);
    expect(ends.right.y_cm).toBeCloseTo(25);
    expect(ends.left.y_cm).toBeCloseTo(-25);
  });
});
