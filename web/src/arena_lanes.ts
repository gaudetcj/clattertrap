// What the arena page opens with and the lanes it plays: one gallery entry per model for a level,
// an arm, and a run number.

import type { LaneInput } from './conductor';
import { lineup, modelLabel, type Entry } from './gallery';
import type { Fixture } from './levels';
import { param } from './query';

export const DEFAULT_LEVEL = 'seesaw-launch';
export const DEFAULT_ARM = 'blueprint';

export interface Pick {
  level: string;
  arm: string;
  run?: string;
}

// The query's level, arm, and run, or the hero defaults.
export function pickFrom(query: string): Pick {
  const pick: Pick = { level: param(query, 'level') ?? DEFAULT_LEVEL, arm: param(query, 'arm') ?? DEFAULT_ARM };
  const run = param(query, 'run');
  if (run !== undefined) {
    pick.run = run;
  }
  return pick;
}

export function lanesFor(entries: Entry[], levels: Fixture[], pick: Pick): LaneInput[] {
  const level = levels.find((f) => f.id === pick.level);
  if (!level) {
    return [];
  }
  return lineup(entries, pick.level, pick.arm, pick.run).map((e) => ({
    chrome: { label: modelLabel(e.model), name: e.name, prediction: e.prediction },
    levelJson: level.level,
    sceneJson: e.sceneJson,
    stored: e.outcome,
  }));
}
