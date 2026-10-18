// The pilot runs bundled into the dev page: every submission under pilot/runs, keyed by its run path,
// so a replay is one pick from a list or one link instead of a file drop.

import type { GoalKind } from './decode';
import { decodeOutcome } from './decode';
import { stampText } from './stamp';

export interface Run {
  key: string;
  label: string;
  text: string;
}

const SUBMISSION = /pilot\/runs\/([^/]+)\/([^/]+)\/([^/]+)\/submission\.json$/;

// `claude-opus-5-5/seesaw-launch/blueprint-01` from a glob path ending in that run's submission.json.
export function runKey(path: string): string | undefined {
  const m = SUBMISSION.exec(path);
  return m ? `${m[1]}/${m[2]}/${m[3]}` : undefined;
}

// The verdict stamp for a submission record, or nothing when the record cannot be read.
function verdict(text: string, goalOf: (levelId: string) => GoalKind | undefined): string | undefined {
  try {
    const parsed: unknown = JSON.parse(text);
    if (typeof parsed !== 'object' || parsed === null) {
      return undefined;
    }
    const record = parsed as { level?: unknown; outcome?: unknown };
    const goal = typeof record.level === 'string' ? goalOf(record.level) : undefined;
    if (goal === undefined) {
      return undefined;
    }
    return stampText(goal, decodeOutcome(JSON.stringify(record.outcome)));
  } catch {
    return undefined;
  }
}

// Every submission in the globbed files, sorted by key, each labelled `model level run  VERDICT`.
export function indexRuns(
  files: Record<string, string>,
  goalOf: (levelId: string) => GoalKind | undefined,
): Run[] {
  const runs: Run[] = [];
  for (const [path, text] of Object.entries(files)) {
    const key = runKey(path);
    if (key === undefined) {
      continue;
    }
    const stamp = verdict(text, goalOf);
    const label = stamp === undefined ? key.replaceAll('/', ' ') : `${key.replaceAll('/', ' ')}  ${stamp}`;
    runs.push({ key, label, text });
  }
  return runs.sort((a, b) => a.key.localeCompare(b.key));
}

// The run named by a page's `?run=` query, if any.
export function runFromSearch(search: string): string | undefined {
  const run = new URLSearchParams(search).get('run');
  return run === null || run === '' ? undefined : run;
}

// The query string that opens a run.
export function searchForRun(key: string): string {
  return `?run=${encodeURIComponent(key)}`;
}
