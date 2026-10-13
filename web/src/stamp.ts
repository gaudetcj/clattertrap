// The verdict stamped over a finished machine. Success names the goal and the time; failure names why.

import type { GoalKind, Outcome } from './decode';

const VERBS: Record<GoalKind, string> = {
  ring: 'RANG',
  press: 'PRESSED',
  fill: 'FILLED',
  pop: 'POPPED',
};

export function stampText(goal: GoalKind, outcome: Outcome): string {
  switch (outcome.end) {
    case 'goal_met':
      return `${VERBS[goal]} ${outcome.time_s.toFixed(1)} s`;
    case 'time_limit':
      return 'MISSED';
    case 'all_asleep':
      return 'CAME TO REST';
  }
}
