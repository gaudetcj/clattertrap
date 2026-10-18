// The stamp a panel ends on. The viewer never decides an outcome: when a stored result exists the
// stamp repeats it, and a replay that disagrees with it is stamped as a mismatch instead.

import type { GoalKind, Outcome } from './decode';
import { stampText } from './stamp';

export type Tone = 'success' | 'failure' | 'mismatch';

export interface Stamp {
  text: string;
  tone: Tone;
}

export const MISMATCH = 'REPLAY MISMATCH';

const COLOURS: Record<Tone, string> = {
  success: '#1f7a3a',
  failure: '#b3261e',
  mismatch: '#e2b53c',
};

export function toneColour(tone: Tone): string {
  return COLOURS[tone];
}

// Whether a replay reproduced a stored outcome: the same hashes, end, length, verdict, and rules.
export function outcomesAgree(stored: Outcome, replayed: Outcome): boolean {
  return (
    stored.event_hash === replayed.event_hash &&
    stored.state_hash === replayed.state_hash &&
    stored.end === replayed.end &&
    stored.steps === replayed.steps &&
    stored.goal_met === replayed.goal_met &&
    stored.rules_version === replayed.rules_version
  );
}

export function verdictFor(goal: GoalKind, replayed: Outcome, stored: Outcome | null): Stamp {
  if (stored === null) {
    return { text: stampText(goal, replayed), tone: replayed.goal_met ? 'success' : 'failure' };
  }
  if (!outcomesAgree(stored, replayed)) {
    return { text: MISMATCH, tone: 'mismatch' };
  }
  return { text: stampText(goal, stored), tone: stored.goal_met ? 'success' : 'failure' };
}
