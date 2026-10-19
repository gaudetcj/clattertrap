// One line on how the models did on a level blind, for a visitor who just played it.

import type { Goal, GoalKind } from './decode';
import { modelLabel, type Entry } from './gallery';

const PAST: Record<GoalKind, string> = {
  ring: 'rang',
  press: 'pressed',
  fill: 'filled',
  pop: 'popped',
};

export function modelsLine(entries: Entry[], level: string, goal: GoalKind): string {
  const blind = entries.filter((e) => e.level === level && e.arm === 'blueprint');
  const models = new Set(blind.map((e) => e.model));
  if (models.size === 0) {
    return 'No model has played this level yet.';
  }
  const solved = blind.filter((e) => e.outcome.goal_met);
  const head = `${new Set(solved.map((e) => e.model)).size} of ${models.size} models ${PAST[goal]} it blind`;
  const fastest = solved.sort((a, b) => a.outcome.time_s - b.outcome.time_s)[0];
  if (!fastest) {
    return `${head}.`;
  }
  return `${head}; the fastest was ${modelLabel(fastest.model)} in ${fastest.outcome.time_s.toFixed(1)} s.`;
}

const VERB: Record<GoalKind, string> = {
  ring: 'Ring',
  press: 'Press',
  fill: 'Fill',
  pop: 'Pop',
};

// The goal as a sentence for the panel's footer.
export function goalLine(goal: Goal): string {
  let line = `${VERB[goal.kind]} the ${goal.target}`;
  if (goal.kind === 'fill' && goal.count !== undefined) {
    line += ` with ${goal.count} things`;
  }
  if (goal.via.length > 0) {
    line += `, touching ${goal.via.join(' then ')} on the way`;
  }
  return `${line}.`;
}
