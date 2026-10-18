import { describe, expect, it } from 'vitest';
import { indexRuns, runFromSearch, runKey, searchForRun } from './runs';

const RANG = JSON.stringify({
  level: 'seesaw-launch',
  arm: 'blueprint',
  name: 'Swatter Seesaw',
  prediction: 'The arm swings up and rings the bell.',
  scene: { level: 'seesaw-launch', parts: [] },
  outcome: { goal_met: true, end: 'goal_met', time_s: 0.925, steps: 111, parts_used: 1, chain_length: 1, event_hash: '0', state_hash: '0', rules_version: 1 },
});
const MISSED = JSON.stringify({
  level: 'domino-run',
  arm: 'blueprint',
  name: 'Long row',
  prediction: 'The dominoes fall and press the button.',
  scene: { level: 'domino-run', parts: [] },
  outcome: { goal_met: false, end: 'time_limit', time_s: 20, steps: 2400, parts_used: 8, chain_length: 0, event_hash: '0', state_hash: '0', rules_version: 1 },
});

describe('runKey', () => {
  it('is the model, level, and run directory of a submission path', () => {
    expect(runKey('../../pilot/runs/claude-opus-5-5/seesaw-launch/blueprint-01/submission.json')).toBe(
      'claude-opus-5-5/seesaw-launch/blueprint-01',
    );
  });

  it('is undefined for any other path', () => {
    expect(runKey('../../pilot/runs/claude-opus-5-5/seesaw-launch/blueprint-01/summary.json')).toBeUndefined();
    expect(runKey('../../levels/bell-drop/level.json')).toBeUndefined();
  });
});

describe('indexRuns', () => {
  const goalOf = (levelId: string): 'ring' | 'press' | undefined =>
    levelId === 'seesaw-launch' ? 'ring' : levelId === 'domino-run' ? 'press' : undefined;

  it('lists runs sorted by key with the verdict in the label', () => {
    const runs = indexRuns(
      {
        '../../pilot/runs/claude-sonnet-5-5/domino-run/blueprint-01/submission.json': MISSED,
        '../../pilot/runs/claude-opus-5-5/seesaw-launch/blueprint-01/submission.json': RANG,
        '../../pilot/runs/claude-opus-5-5/seesaw-launch/blueprint-01/summary.json': '{}',
      },
      goalOf,
    );
    expect(runs.map((r) => r.key)).toEqual([
      'claude-opus-5-5/seesaw-launch/blueprint-01',
      'claude-sonnet-5-5/domino-run/blueprint-01',
    ]);
    expect(runs[0]?.label).toBe('claude-opus-5-5 seesaw-launch blueprint-01  RANG 0.9 s');
    expect(runs[1]?.label).toBe('claude-sonnet-5-5 domino-run blueprint-01  MISSED');
    expect(runs[0]?.text).toBe(RANG);
  });

  it('labels a submission it cannot read by its key alone', () => {
    const runs = indexRuns({ '../../pilot/runs/m/l/workshop-02/submission.json': '{ not json' }, goalOf);
    expect(runs[0]?.label).toBe('m l workshop-02');
  });
});

describe('the run query', () => {
  it('round-trips a key through the page query string', () => {
    const search = searchForRun('claude-opus-5-5/seesaw-launch/blueprint-01');
    expect(search).toBe('?run=claude-opus-5-5%2Fseesaw-launch%2Fblueprint-01');
    expect(runFromSearch(search)).toBe('claude-opus-5-5/seesaw-launch/blueprint-01');
    expect(runFromSearch('?run=claude-opus-5-5/seesaw-launch/blueprint-01')).toBe('claude-opus-5-5/seesaw-launch/blueprint-01');
  });

  it('is undefined without a run', () => {
    expect(runFromSearch('')).toBeUndefined();
    expect(runFromSearch('?level=bell-drop')).toBeUndefined();
  });
});
