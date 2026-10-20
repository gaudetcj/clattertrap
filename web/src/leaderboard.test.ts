import { describe, expect, it } from 'vitest';
import type { Entry, Miss } from './gallery';
import { countLine, leaderboardRows, sortRows } from './leaderboard';

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

  it('rank by blueprint solved, then workshop solved, then mean cost, whatever the coverage', () => {
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
    expect(rows.map((r) => r.model)).toEqual(['partial', 'cheap', 'pricey']);
    expect(rows[0]?.full).toBe(false);
    expect(rows[1]?.full).toBe(true);
  });

  it('sort by any column, descending for scores and ascending for cost and name, and flip on request', () => {
    const rows = leaderboardRows(
      [
        entry('a', 'bell-drop', 'blueprint', '01', true, 1),
        entry('a', 'post-first', 'workshop', '01', false, 1),
        entry('b', 'bell-drop', 'workshop', '01', true, 3),
        entry('b', 'seesaw-launch', 'workshop', '01', true, 3),
        entry('c', 'seesaw-launch', 'blueprint', '01', true, 0.5),
      ],
      [],
      tiers,
    );
    const names = (sorted: typeof rows): string[] => sorted.map((r) => r.model);
    expect(names(rows)).toEqual(['c', 'a', 'b']);
    expect(names(sortRows(rows, 'workshop', false))).toEqual(['b', 'c', 'a']);
    expect(names(sortRows(rows, 'hardBlueprint', false))).toEqual(['c', 'a', 'b']);
    expect(names(sortRows(rows, 'meanUsd', false))).toEqual(['c', 'a', 'b']);
    expect(names(sortRows(rows, 'model', false))).toEqual(['a', 'b', 'c']);
    expect(names(sortRows(rows, 'runs', false))).toEqual(['a', 'b', 'c']);
    expect(names(sortRows(rows, 'model', true))).toEqual(['c', 'b', 'a']);
    expect(names(sortRows(rows, 'blueprint', true))).toEqual(['b', 'a', 'c']);
  });
});

describe('count line', () => {
  it('ties the counted runs to the machines built and the runs without a submission', () => {
    expect(countLine(339, 316, 11, 1, 15)).toBe(
      '339 counted runs by 11 models on rules version 1, 316 of them with a machine and 23 without a submission. ' +
        'A level counts once however many runs solved it, over the levels the model attempted in that arm, of 15; hard levels are tiers 4 and 5.',
    );
  });
});
