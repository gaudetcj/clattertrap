// The leaderboard: levels solved per model and arm over the gallery, ranked the way
// `clattertrap leaderboard` ranks them: blueprint levels solved, then workshop levels solved, mean
// cost, and id. A level counts once however many runs solved it. The page can sort by any column.

import type { Entry, Miss } from './gallery';

// Tiers 4 and 5 are hard; see docs/levels.md.
export const HARD_TIER = 4;

export interface ArmCount {
  solved: number;
  attempted: number;
  hardSolved: number;
  hardAttempted: number;
}

export interface Row {
  model: string;
  blueprint: ArmCount;
  workshop: ArmCount;
  meanUsd: number;
  runs: number;
  // Whether the model attempted every level under both arms.
  full: boolean;
}

// A column the page can sort by.
export type SortKey = 'model' | 'blueprint' | 'workshop' | 'hardBlueprint' | 'hardWorkshop' | 'meanUsd' | 'runs';

interface Counted {
  model: string;
  level: string;
  arm: string;
  solved: boolean;
  cost: number | null;
}

// Every counted run: the submissions, then the counted runs without one.
function counted(entries: Entry[], misses: Miss[]): Counted[] {
  return [
    ...entries.map((e) => ({ model: e.model, level: e.level, arm: e.arm, solved: e.outcome.goal_met, cost: e.summary?.cost_usd ?? null })),
    ...misses.map((m) => ({ model: m.model, level: m.level, arm: m.arm, solved: false, cost: m.cost_usd })),
  ];
}

function armCount(runs: Counted[], arm: string, tiers: Map<string, number>): ArmCount {
  const mine = runs.filter((r) => r.arm === arm);
  const attempted = new Set(mine.map((r) => r.level));
  const solved = new Set(mine.filter((r) => r.solved).map((r) => r.level));
  const hard = (ids: Set<string>): number => [...ids].filter((id) => (tiers.get(id) ?? 0) >= HARD_TIER).length;
  return { solved: solved.size, attempted: attempted.size, hardSolved: hard(solved), hardAttempted: hard(attempted) };
}

function mean(values: number[]): number {
  return values.length === 0 ? 0 : values.reduce((a, b) => a + b, 0) / values.length;
}

// The ranking: more blueprint levels solved, then more workshop levels, then cheaper, then by id.
function rank(a: Row, b: Row): number {
  return b.blueprint.solved - a.blueprint.solved || b.workshop.solved - a.workshop.solved || a.meanUsd - b.meanUsd || a.model.localeCompare(b.model);
}

export function leaderboardRows(entries: Entry[], misses: Miss[], tiers: Map<string, number>): Row[] {
  const runs = counted(entries, misses);
  const total = tiers.size;
  const rows = [...new Set(runs.map((r) => r.model))].map((model): Row => {
    const mine = runs.filter((r) => r.model === model);
    const blueprint = armCount(mine, 'blueprint', tiers);
    const workshop = armCount(mine, 'workshop', tiers);
    return {
      model,
      blueprint,
      workshop,
      meanUsd: mean(mine.map((r) => r.cost).filter((c): c is number => c !== null)),
      runs: mine.length,
      full: blueprint.attempted === total && workshop.attempted === total,
    };
  });
  return rows.sort(rank);
}

// Columns whose natural order puts the largest first; cost and id read upward.
const LARGEST_FIRST: ReadonlySet<SortKey> = new Set(['blueprint', 'workshop', 'hardBlueprint', 'hardWorkshop', 'runs']);

function valueOf(r: Row, key: SortKey): number | string {
  switch (key) {
    case 'model':
      return r.model;
    case 'blueprint':
      return r.blueprint.solved;
    case 'workshop':
      return r.workshop.solved;
    case 'hardBlueprint':
      return r.blueprint.hardSolved;
    case 'hardWorkshop':
      return r.workshop.hardSolved;
    case 'meanUsd':
      return r.meanUsd;
    case 'runs':
      return r.runs;
  }
}

// The rows ordered by one column in its natural direction, ties broken by the ranking; `flipped`
// reverses the whole order.
export function sortRows(rows: Row[], key: SortKey, flipped: boolean): Row[] {
  const order = (a: Row, b: Row): number => {
    const x = valueOf(a, key);
    const y = valueOf(b, key);
    const cmp = typeof x === 'string' && typeof y === 'string' ? x.localeCompare(y) : Number(x) - Number(y);
    return (LARGEST_FIRST.has(key) ? -cmp : cmp) || rank(a, b);
  };
  return [...rows].sort((a, b) => (flipped ? -order(a, b) : order(a, b)));
}

// The line under the table, as `clattertrap leaderboard --markdown` writes it for the README.
export function countLine(runs: number, machines: number, models: number, version: number, total: number): string {
  return (
    `${runs} counted runs by ${models} models on rules version ${version}, ${machines} of them with a machine and ${runs - machines} without a submission. ` +
    `A level counts once however many runs solved it, over the levels the model attempted in that arm, of ${total}; hard levels are tiers ${HARD_TIER} and 5.`
  );
}
