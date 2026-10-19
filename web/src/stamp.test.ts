import { describe, expect, it } from 'vitest';
import type { Outcome } from './decode';
import { missStamp, stampText } from './stamp';

function outcome(end: Outcome['end'], time_s: number): Outcome {
  return {
    goal_met: end === 'goal_met',
    end,
    time_s,
    steps: Math.round(time_s * 120),
    parts_used: 2,
    chain_length: 2,
    event_hash: '0',
    state_hash: '0',
    rules_version: 1,
  };
}

describe('stampText', () => {
  it('names the goal and the time when it was met', () => {
    expect(stampText('ring', outcome('goal_met', 1.925))).toBe('RANG 1.9 s');
    expect(stampText('press', outcome('goal_met', 4.2))).toBe('PRESSED 4.2 s');
    expect(stampText('fill', outcome('goal_met', 12.5))).toBe('FILLED 12.5 s');
    expect(stampText('pop', outcome('goal_met', 3))).toBe('POPPED 3.0 s');
  });

  it('says MISSED at the time limit and CAME TO REST when everything slept', () => {
    expect(stampText('ring', outcome('time_limit', 20))).toBe('MISSED');
    expect(stampText('fill', outcome('all_asleep', 6.1))).toBe('CAME TO REST');
  });
});

describe('missStamp', () => {
  it('names a counted run that ended without a submission', () => {
    expect(missStamp('no_submission')).toBe('NO SUBMISSION');
    expect(missStamp('over_budget')).toBe('OVER BUDGET');
    expect(missStamp('timed_out')).toBe('TIMED OUT');
  });
});
