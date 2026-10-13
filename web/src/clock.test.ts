import { describe, expect, it } from 'vitest';
import { stepsDue } from './clock';

const DT = 1 / 120;

describe('stepsDue', () => {
  it('owes the steps that wall time has passed, less those done', () => {
    expect(stepsDue(0, 0, DT)).toBe(0);
    expect(stepsDue(1 / 60, 0, DT)).toBe(2);
    expect(stepsDue(1 / 60, 1, DT)).toBe(1);
    expect(stepsDue(0.5, 60, DT)).toBe(0);
  });

  it('never owes more than the catch-up cap so a stalled tab does not freeze', () => {
    expect(stepsDue(5, 0, DT)).toBe(6);
    expect(stepsDue(5, 0, DT, 10)).toBe(10);
  });

  it('never goes negative', () => {
    expect(stepsDue(0.1, 50, DT)).toBe(0);
  });
});
