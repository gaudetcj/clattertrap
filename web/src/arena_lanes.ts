// What the arena page opens with and the lanes it plays: one gallery entry per model for a level,
// an arm, and a run number, in leaderboard order, cut down to a lineup when many models played.

import type { LaneInput } from './conductor';
import { familyOf, lineup, modelLabel, type Entry, type Miss } from './gallery';
import type { Fixture } from './levels';
import { param } from './query';
import { missStamp } from './stamp';

export const DEFAULT_LEVEL = 'seesaw-launch';
export const DEFAULT_ARM = 'blueprint';

// The named lineups: the strongest of each family, every model, the Claude models, or the
// open-weights models.
export type LineupKind = 'families' | 'all' | 'claude' | 'open';
export const LINEUPS: readonly LineupKind[] = ['families', 'all', 'claude', 'open'];

// A level with more models than this opens on one per family.
const FULL_GRID = 6;

// Families whose weights are open.
const OPEN_FAMILIES = ['deepseek', 'z-ai', 'xiaomi', 'qwen', 'meta-llama', 'mistralai', 'nvidia'];

export interface Pick {
  level: string;
  arm: string;
  run?: string;
  lineup?: LineupKind;
  models?: string[];
}

function isLineup(v: string): v is LineupKind {
  return (LINEUPS as readonly string[]).includes(v);
}

// The query's level, arm, run, lineup, and model list, or the hero defaults.
export function pickFrom(query: string): Pick {
  const pick: Pick = { level: param(query, 'level') ?? DEFAULT_LEVEL, arm: param(query, 'arm') ?? DEFAULT_ARM };
  const run = param(query, 'run');
  if (run !== undefined) {
    pick.run = run;
  }
  const kind = param(query, 'lineup');
  if (kind !== undefined && isLineup(kind)) {
    pick.lineup = kind;
  }
  const models = (param(query, 'models') ?? '').split(',').filter((m) => m.length > 0);
  if (models.length > 0) {
    pick.models = models;
  }
  return pick;
}

export function defaultLineup(present: number): LineupKind {
  return present > FULL_GRID ? 'families' : 'all';
}

// The models a lineup shows: `ranked` is the leaderboard order and `present` the models with an
// entry on the level and arm.
export function lineupModels(kind: LineupKind, ranked: string[], present: string[]): string[] {
  const here = ranked.filter((m) => present.includes(m));
  switch (kind) {
    case 'all':
      return here;
    case 'claude':
      return here.filter((m) => familyOf(m) === 'claude');
    case 'open':
      return here.filter((m) => OPEN_FAMILIES.includes(familyOf(m)));
    case 'families': {
      const seen = new Set<string>();
      return here.filter((m) => {
        const family = familyOf(m);
        if (seen.has(family)) {
          return false;
        }
        seen.add(family);
        return true;
      });
    }
  }
}

// What a model has on a level and arm: its first entry, or, with none, its first counted run that
// ended without a submission. A run number narrows both to that run.
type Played = { entry: Entry; miss?: undefined } | { entry?: undefined; miss: Miss };

function played(entries: Entry[], misses: Miss[], pick: Pick): Map<string, Played> {
  const out = new Map<string, Played>();
  for (const e of lineup(entries, pick.level, pick.arm, pick.run)) {
    out.set(e.model, { entry: e });
  }
  const mine = misses
    .filter((m) => m.level === pick.level && m.arm === pick.arm && (pick.run === undefined || m.run === `${pick.arm}-${pick.run}`))
    .sort((a, b) => a.run.localeCompare(b.run));
  for (const m of mine) {
    if (!out.has(m.model)) {
      out.set(m.model, { miss: m });
    }
  }
  return out;
}

// The models with a counted run on the level and arm, a submission or not.
export function presentModels(entries: Entry[], misses: Miss[], pick: Pick): string[] {
  return [...played(entries, misses, pick).keys()];
}

// The models the pick plays, in lane order: a custom list as given, else the named or default
// lineup over the models present.
export function pickModels(entries: Entry[], misses: Miss[], pick: Pick, ranked: string[]): string[] {
  const present = presentModels(entries, misses, pick);
  if (pick.models) {
    return pick.models.filter((m) => present.includes(m));
  }
  return lineupModels(pick.lineup ?? defaultLineup(present.length), ranked, present);
}

// Why a counted run has no machine, for the panel's footer; the wording follows docs/format.md.
function missNote(status: string): string {
  switch (status) {
    case 'no_submission':
      return 'The session ended on its own without a submission.';
    case 'over_budget':
      return 'The dollar cap stopped the session before a submission.';
    case 'timed_out':
      return 'The wall clock stopped the session before a submission.';
    default:
      return `The session ended ${status.replace(/_/g, ' ')} without a submission.`;
  }
}

// A lane for a counted run without a submission: the level as the model left it, nothing placed.
function missLane(m: Miss, level: Fixture): LaneInput {
  return {
    chrome: { label: modelLabel(m.model), name: '', prediction: missNote(m.status) },
    levelJson: level.level,
    sceneJson: JSON.stringify({ level: level.id, parts: [] }),
    stored: null,
    missing: { text: missStamp(m.status), tone: 'failure' },
  };
}

export function lanesFor(entries: Entry[], misses: Miss[], levels: Fixture[], pick: Pick, ranked: string[]): LaneInput[] {
  const level = levels.find((f) => f.id === pick.level);
  if (!level) {
    return [];
  }
  const byModel = played(entries, misses, pick);
  return pickModels(entries, misses, pick, ranked).flatMap((m) => {
    const p = byModel.get(m);
    if (p?.entry) {
      const e = p.entry;
      return [
        {
          chrome: { label: modelLabel(e.model), name: e.name, prediction: e.prediction },
          levelJson: level.level,
          sceneJson: e.sceneJson,
          stored: e.outcome,
        },
      ];
    }
    return p ? [missLane(p.miss, level)] : [];
  });
}

// The level after `current` in `ids`, wrapping to the first; an unknown current starts over.
export function nextLevelId(ids: string[], current: string): string {
  if (ids.length === 0) {
    return current;
  }
  const at = ids.indexOf(current);
  return ids[(at + 1) % ids.length] ?? current;
}

// What the lineup dropdown calls each kind, with how many of the models present it plays.
export function lineupNames(ranked: string[], present: string[]): Record<LineupKind, string> {
  const count = (kind: LineupKind): number => lineupModels(kind, ranked, present).length;
  return {
    families: `one per family (${count('families')})`,
    all: `all models (${count('all')})`,
    claude: `Claude (${count('claude')})`,
    open: `open weights (${count('open')})`,
  };
}
