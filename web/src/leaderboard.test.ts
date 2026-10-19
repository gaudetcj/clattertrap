import { describe, expect, it } from 'vitest';
import type { Entry, Miss } from './gallery';
import { leaderboardRows } from './leaderboard';

function entry(model: string, level: string, arm: string, run: string, goal_met: boolean, cost: number | null): Entry {
  return {
    key: `${level}/${model}/${arm}-${run}`,
    level,
    model,
    arm,
    run: `${arm}-${run}`,
    name: 'm',
    prediction: '',
    sceneJson: '{}',
    outcome: {
      goal_met,
      end: goal_met ? 'goal_met' : 'time_limit',
      time_s: 1,
      steps: 1,
      parts_used: 1,
      chain_length: 1,
      event_hash: '0',
      state_hash: '0',
      rules_version: 1,
    },
    summary: { status: 'submitted', cost_usd: cost, duration_s: 1 },
  };
}

function miss(model: string, level: string, arm: string, cost: number | null): Miss {
  return { key: `${level}/${model}/${arm}-01`, level, model, run: `${arm}-01`, arm, status: 'no_submission', cost_usd: cost };
}

const tiers = new Map<string, number>([
  ['bell-drop', 1],
  ['post-first', 3],
  ['seesaw-launch', 4],
]);

describe('leaderboard rows', () => {
  it('count a level once, count a miss as attempted, and average the cost over every counted run', () => {
    const rows = leaderboardRows(
      [
        entry('m', 'bell-drop', 'workshop', '01', true, 1),
        entry('m', 'bell-drop', 'workshop', '02', false, 3),
        entry('m', 'seesaw-launch', 'blueprint', '01', true, null),
      ],
      [miss('m', 'post-first', 'workshop', 2)],
      tiers,
    );
    expect(rows).toHaveLength(1);
    const row = rows[0];
    expect(row?.workshop).toEqual({ solved: 1, attempted: 2, hardSolved: 0, hardAttempted: 0 });
    expect(row?.blueprint).toEqual({ solved: 1, attempted: 1, hardSolved: 1, hardAttempted: 1 });
    expect(row?.runs).toBe(4);
    expect(row?.meanUsd).toBe(2);
    expect(row?.full).toBe(false);
  });

  it('rank full coverage first, then blueprint solved, workshop solved, and mean cost', () => {
    const full = (model: string, cost: number): Entry[] =>
      [...tiers.keys()].flatMap((level) => [
        entry(model, level, 'blueprint', '01', level === 'bell-drop', cost),
        entry(model, level, 'workshop', '01', false, cost),
      ]);
    const rows = leaderboardRows(
      [
        ...full('pricey', 2),
        ...full('cheap', 0.5),
        entry('partial', 'bell-drop', 'blueprint', '01', true, 0.1),
        entry('partial', 'post-first', 'blueprint', '01', true, 0.1),
      ],
      [],
      tiers,
    );
    expect(rows.map((r) => r.model)).toEqual(['cheap', 'pricey', 'partial']);
    expect(rows[0]?.full).toBe(true);
    expect(rows[2]?.blueprint.solved).toBe(2);
  });
});
