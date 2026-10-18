import { describe, expect, it } from 'vitest';
import type { Entry } from './gallery';
import { notesFrom, parseCall, reelSteps } from './reel_plan';

const outcome = {
  goal_met: false,
  end: 'all_asleep',
  time_s: 3,
  steps: 360,
  parts_used: 1,
  chain_length: 1,
  event_hash: '0000000000000001',
  state_hash: '0000000000000002',
  rules_version: 1,
};

function call(seq: number, tool: string, extra: Record<string, unknown>): string {
  return JSON.stringify({ seq, tool, scene: { level: 'bell-drop', parts: [{ id: `p${seq}`, kind: 'ball', x_cm: 1, y_cm: 2, material: 'steel' }] }, ...extra });
}

const entry: Entry = {
  key: 'bell-drop/claude-haiku-5-5/workshop-01',
  level: 'bell-drop',
  model: 'claude-haiku-5-5',
  arm: 'workshop',
  run: 'workshop-01',
  name: 'Two planks',
  prediction: 'It rings.',
  sceneJson: '{"level":"bell-drop","parts":[]}',
  outcome: { ...outcome, goal_met: true, end: 'goal_met' },
};

describe('parseCall', () => {
  it('reads a run call with its outcome and a rejected one without', () => {
    const run = parseCall(call(2, 'run', { outcome }));
    expect(run).toEqual({ seq: 2, tool: 'run', sceneJson: expect.stringContaining('"p2"'), accepted: true, outcome });
    const rejected = parseCall(call(3, 'run', { problems: ['parts[0] (ball p3): overlaps floor'] }));
    expect(rejected?.accepted).toBe(false);
    expect(rejected?.outcome).toBeNull();
    expect(parseCall('{ nope')).toBeUndefined();
    expect(parseCall('{"seq":1,"tool":"check"}')).toBeUndefined();
  });
});

describe('reelSteps', () => {
  it('plays accepted run calls in order, numbered among every run call, then the submission', () => {
    const calls = [
      parseCall(call(6, 'run', { outcome })),
      parseCall(call(1, 'check', {})),
      parseCall(call(2, 'run', { outcome })),
      parseCall(call(4, 'run', { problems: ['no'] })),
      parseCall(call(7, 'submit', { outcome, name: 'Two planks', prediction: 'It rings.' })),
    ].flatMap((c) => (c ? [c] : []));
    const steps = reelSteps(calls, entry);
    expect(steps.map((s) => s.label)).toEqual(['ATTEMPT 1 OF 3', 'ATTEMPT 3 OF 3', 'SUBMISSION']);
    expect(steps[0]?.sceneJson).toContain('"p2"');
    expect(steps[0]?.stored).toEqual(outcome);
    expect(steps[0]?.name).toBe('attempt 1');
    expect(steps[2]?.name).toBe('Two planks');
    expect(steps[2]?.prediction).toBe('It rings.');
    expect(steps[2]?.stored?.goal_met).toBe(true);
  });

  it('is just the submission for a run without accepted attempts', () => {
    const steps = reelSteps([], entry);
    expect(steps.map((s) => s.label)).toEqual(['SUBMISSION']);
  });

  it('is the attempts alone when the run never submitted', () => {
    const steps = reelSteps([parseCall(call(2, 'run', { outcome }))].flatMap((c) => (c ? [c] : [])), null);
    expect(steps.map((s) => s.label)).toEqual(['ATTEMPT 1 OF 1']);
  });
});

describe('notesFrom', () => {
  it('keeps the text records of a gallery transcript in order', () => {
    const jsonl = [
      '{"type":"tool_use","name":"mcp__clattertrap__level","input":{"id":"bell-drop"}}',
      '{"type":"text","text":"Looking at the level."}',
      'not json',
      '{"type":"tool_result","content":[{"type":"text","text":"bell-drop: Ring the bell"}]}',
      '{"type":"text","text":"Submitting two planks."}',
      '{"type":"result","subtype":"success"}',
    ].join('\n');
    expect(notesFrom(jsonl)).toEqual(['Looking at the level.', 'Submitting two planks.']);
    expect(notesFrom('')).toEqual([]);
  });
});
