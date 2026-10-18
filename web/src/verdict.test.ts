import { describe, expect, it } from 'vitest';
import type { Outcome } from './decode';
import { MISMATCH, outcomesAgree, toneColour, verdictFor } from './verdict';

const rang: Outcome = {
  goal_met: true,
  end: 'goal_met',
  time_s: 1.925,
  steps: 231,
  parts_used: 2,
  chain_length: 2,
  event_hash: '41a3e9905cb8f870',
  state_hash: 'bbf5aaffac928e50',
  rules_version: 1,
};

const rested: Outcome = {
  ...rang,
  goal_met: false,
  end: 'all_asleep',
  time_s: 6.5,
  steps: 780,
  event_hash: '0000000000000001',
  state_hash: '0000000000000002',
};

describe('outcomesAgree', () => {
  it('holds for equal outcomes', () => {
    expect(outcomesAgree(rang, { ...rang })).toBe(true);
  });

  it('fails on any hash, the end, the step count, the goal, or the rules version', () => {
    expect(outcomesAgree(rang, { ...rang, event_hash: '41a3e9905cb8f871' })).toBe(false);
    expect(outcomesAgree(rang, { ...rang, state_hash: '0' })).toBe(false);
    expect(outcomesAgree(rang, { ...rang, end: 'time_limit' })).toBe(false);
    expect(outcomesAgree(rang, { ...rang, steps: 230 })).toBe(false);
    expect(outcomesAgree(rang, { ...rang, goal_met: false })).toBe(false);
    expect(outcomesAgree(rang, { ...rang, rules_version: 2 })).toBe(false);
  });
});

describe('verdictFor', () => {
  it('stamps the replay when nothing is stored', () => {
    expect(verdictFor('ring', rang, null)).toEqual({ text: 'RANG 1.9 s', tone: 'success' });
    expect(verdictFor('fill', rested, null)).toEqual({ text: 'CAME TO REST', tone: 'failure' });
  });

  it('stamps the stored outcome when the replay agrees with it', () => {
    expect(verdictFor('press', { ...rang }, rang)).toEqual({ text: 'PRESSED 1.9 s', tone: 'success' });
  });

  it('stamps a mismatch when the replay disagrees, whatever either says', () => {
    expect(verdictFor('ring', rested, rang)).toEqual({ text: MISMATCH, tone: 'mismatch' });
    expect(verdictFor('ring', rang, { ...rang, state_hash: 'ffffffffffffffff' })).toEqual({
      text: MISMATCH,
      tone: 'mismatch',
    });
  });

  it('colours success green, failure red, and a mismatch amber', () => {
    expect(toneColour('success')).toBe('#1f7a3a');
    expect(toneColour('failure')).toBe('#b3261e');
    expect(toneColour('mismatch')).toBe('#e2b53c');
  });
});
