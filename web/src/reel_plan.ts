// The attempt reel: one entry's accepted `run` calls in order, then its submission, each with the
// stored outcome it must replay to. Rejected calls are skipped but still counted in the labels.

import { decodeOutcome, type Outcome } from './decode';
import type { Entry } from './gallery';

export interface Call {
  seq: number;
  tool: string;
  sceneJson: string;
  accepted: boolean;
  outcome: Outcome | null;
}

export interface ReelStep {
  label: string;
  name: string;
  prediction: string;
  sceneJson: string;
  stored: Outcome | null;
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

// A call record, or nothing when the text is not one.
export function parseCall(text: string): Call | undefined {
  try {
    const v: unknown = JSON.parse(text);
    if (!isRecord(v) || typeof v['seq'] !== 'number' || typeof v['tool'] !== 'string' || v['scene'] === undefined) {
      return undefined;
    }
    const scene = v['scene'];
    return {
      seq: v['seq'],
      tool: v['tool'],
      sceneJson: typeof scene === 'string' ? scene : JSON.stringify(scene),
      accepted: v['problems'] === undefined,
      outcome: v['outcome'] === undefined ? null : decodeOutcome(JSON.stringify(v['outcome'])),
    };
  } catch {
    return undefined;
  }
}

export function reelSteps(calls: Call[], submission: Entry | null): ReelStep[] {
  const runs = calls.filter((c) => c.tool === 'run').sort((a, b) => a.seq - b.seq);
  const steps: ReelStep[] = runs.flatMap((c, i) =>
    c.accepted
      ? [
          {
            label: `ATTEMPT ${i + 1} OF ${runs.length}`,
            name: `attempt ${i + 1}`,
            prediction: '',
            sceneJson: c.sceneJson,
            stored: c.outcome,
          },
        ]
      : [],
  );
  if (submission !== null) {
    steps.push({
      label: 'SUBMISSION',
      name: submission.name,
      prediction: submission.prediction,
      sceneJson: submission.sceneJson,
      stored: submission.outcome,
    });
  }
  return steps;
}

// What the model wrote, in order: the text records of a gallery transcript.
export function notesFrom(transcriptJsonl: string): string[] {
  const notes: string[] = [];
  for (const line of transcriptJsonl.split('\n')) {
    try {
      const v: unknown = JSON.parse(line);
      if (isRecord(v) && v['type'] === 'text' && typeof v['text'] === 'string') {
        notes.push(v['text']);
      }
    } catch {
      // Not a record.
    }
  }
  return notes;
}
