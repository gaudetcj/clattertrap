import { describe, expect, it } from 'vitest';
import { lanesFor, pickFrom, nextLevelId } from './arena_lanes';
import type { Entry } from './gallery';

function entry(model: string, level: string, arm: string, run: string): Entry {
  return {
    key: `${level}/${model}/${run}`,
    level,
    model,
    arm,
    run,
    name: `${model} machine`,
    prediction: 'It works.',
    sceneJson: `{"level":"${level}","parts":[]}`,
    outcome: {
      goal_met: true,
      end: 'goal_met',
      time_s: 1,
      steps: 120,
      parts_used: 1,
      chain_length: 1,
      event_hash: '0000000000000001',
      state_hash: '0000000000000002',
      rules_version: 1,
    },
  };
}

describe('the arena pick', () => {
  it('comes from the query or the hero defaults', () => {
    expect(pickFrom('')).toEqual({ level: 'seesaw-launch', arm: 'blueprint' });
    expect(pickFrom('?level=grand-tour&arm=workshop&run=02')).toEqual({ level: 'grand-tour', arm: 'workshop', run: '02' });
  });
});

describe('lanesFor', () => {
  const levels = [{ id: 'seesaw-launch', level: '{"id":"seesaw-launch"}', reference: '{}' }];
  const entries = [
    entry('claude-opus-5-5', 'seesaw-launch', 'blueprint', 'blueprint-01'),
    entry('claude-haiku-4-5', 'seesaw-launch', 'blueprint', 'blueprint-01'),
    entry('claude-sonnet-5-5', 'seesaw-launch', 'workshop', 'workshop-01'),
  ];

  it('builds one lane per model in leaderboard order with the level JSON and the stored outcome', () => {
    const lanes = lanesFor(entries, [], levels, pickFrom(''), ['claude-opus-5-5', 'claude-haiku-4-5']);
    expect(lanes.map((l) => l.chrome.label)).toEqual(['OPUS 5.5', 'HAIKU 4.5']);
    expect(lanes[0]?.levelJson).toBe('{"id":"seesaw-launch"}');
    expect(lanes[0]?.chrome.name).toBe('claude-opus-5-5 machine');
    expect(lanes[0]?.stored?.goal_met).toBe(true);
  });

  it('is empty for a level the page does not know', () => {
    expect(lanesFor(entries, [], levels, { level: 'nope', arm: 'blueprint' }, [])).toEqual([]);
  });
});

describe('next puzzle', () => {
  it('moves to the following level and wraps around', () => {
    const ids = ['balloon-pin', 'bell-drop', 'seesaw-launch'];
    expect(nextLevelId(ids, 'bell-drop')).toBe('seesaw-launch');
    expect(nextLevelId(ids, 'seesaw-launch')).toBe('balloon-pin');
    expect(nextLevelId(ids, 'unknown')).toBe('balloon-pin');
    expect(nextLevelId([], 'bell-drop')).toBe('bell-drop');
  });
});
