import { describe, expect, it } from 'vitest';
import { marqueeLine, odometer, type Counts } from './chrome';

const counts: Counts = { machines: 93, models: 6, levels: 15, rulesVersion: 1 };

describe('the counter', () => {
  it('pads to the digit count and saturates', () => {
    expect(odometer(93, 6)).toEqual(['0', '0', '0', '0', '9', '3']);
    expect(odometer(0, 4)).toEqual(['0', '0', '0', '0']);
    expect(odometer(1234567, 6)).toEqual(['9', '9', '9', '9', '9', '9']);
  });
});

describe('the marquee', () => {
  it('names the counts and the rules version in one line', () => {
    const line = marqueeLine(counts);
    expect(line).toContain('93 MACHINES');
    expect(line).toContain('6 MODELS');
    expect(line).toContain('15 LEVELS');
    expect(line).toContain('RULES V1');
    expect(line).not.toContain('\n');
  });
});
