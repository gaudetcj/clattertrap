import { describe, expect, it } from 'vitest';
import { byLevel, callKey, entryKey, indexGallery, keyString, lineup, loadersFor, misses, MODEL_ORDER, modelLabel, modelRank, transcriptKey } from './gallery';

function submission(level: string, name: string, goal_met: boolean): string {
  return JSON.stringify({
    level,
    arm: 'blueprint',
    name,
    prediction: `${name} does it.`,
    scene: { level, parts: [] },
    outcome: {
      goal_met,
      end: goal_met ? 'goal_met' : 'time_limit',
      time_s: 0.925,
      steps: 111,
      parts_used: 1,
      chain_length: 1,
      event_hash: '0000000000000001',
      state_hash: '0000000000000002',
      rules_version: 1,
    },
  });
}

const summary = JSON.stringify({ model: 'claude-opus-5-5', level: 'seesaw-launch', arm: 'blueprint', status: 'submitted', started_unix_s: 1, duration_s: 79.1, cost_usd: 0.24, turns: 8, tool_calls: 7, foreign_tools: [], result_subtype: 'success', exit_code: 0, error: null });

describe('entry keys', () => {
  it('come from a submission path, level first', () => {
    expect(entryKey('../../gallery/seesaw-launch/claude-opus-5-5/blueprint-01/submission.json')).toEqual({
      level: 'seesaw-launch',
      model: 'claude-opus-5-5',
      run: 'blueprint-01',
    });
    expect(keyString({ level: 'seesaw-launch', model: 'claude-opus-5-5', run: 'blueprint-01' })).toBe(
      'seesaw-launch/claude-opus-5-5/blueprint-01',
    );
    expect(entryKey('../../gallery/seesaw-launch/claude-opus-5-5/blueprint-01/summary.json')).toBeUndefined();
    expect(entryKey('../../levels/bell-drop/level.json')).toBeUndefined();
  });

  it('name a call by its entry, sequence, and tool', () => {
    expect(callKey('../../gallery/bell-drop/claude-haiku-5-5/workshop-01/calls/0003-run.json')).toEqual({
      key: { level: 'bell-drop', model: 'claude-haiku-5-5', run: 'workshop-01' },
      seq: 3,
      tool: 'run',
    });
    expect(callKey('../../gallery/bell-drop/claude-haiku-5-5/workshop-01/submission.json')).toBeUndefined();
  });
});

describe('models', () => {
  it('are labelled by family and version', () => {
    expect(modelLabel('claude-opus-5-5')).toBe('OPUS 5.5');
    expect(modelLabel('claude-haiku-4-5')).toBe('HAIKU 4.5');
    expect(modelLabel('gpt-5-mini')).toBe('GPT 5.mini');
    expect(modelLabel('llama3')).toBe('LLAMA3');
  });

  it('rank the six Claude models first, in order, and others after by name', () => {
    expect(MODEL_ORDER).toHaveLength(6);
    expect(modelRank('claude-haiku-4-5')).toBe(0);
    expect(modelRank('claude-opus-5-5')).toBe(5);
    expect(modelRank('gpt-5')).toBe(6);
  });
});

describe('indexGallery', () => {
  const files = {
    '../../gallery/seesaw-launch/claude-sonnet-5-5/blueprint-01/submission.json': submission('seesaw-launch', 'Tilted plank', false),
    '../../gallery/seesaw-launch/claude-opus-5-5/blueprint-01/submission.json': submission('seesaw-launch', 'Swatter Seesaw', true),
    '../../gallery/seesaw-launch/claude-opus-5-5/blueprint-02/submission.json': submission('seesaw-launch', 'Second try', true),
    '../../gallery/bell-drop/claude-opus-5-5/workshop-01/submission.json': submission('bell-drop', 'Two planks', true),
    '../../gallery/bell-drop/claude-haiku-5-5/workshop-01/submission.json': '{ not json',
  };
  const summaries = {
    '../../gallery/seesaw-launch/claude-opus-5-5/blueprint-01/summary.json': summary,
  };

  it('sorts entries by key, carries the record, and attaches the summary when there is one', () => {
    const entries = indexGallery(files, summaries);
    expect(entries.map((e) => e.key)).toEqual([
      'bell-drop/claude-opus-5-5/workshop-01',
      'seesaw-launch/claude-opus-5-5/blueprint-01',
      'seesaw-launch/claude-opus-5-5/blueprint-02',
      'seesaw-launch/claude-sonnet-5-5/blueprint-01',
    ]);
    const opus = entries[1];
    expect(opus?.model).toBe('claude-opus-5-5');
    expect(opus?.arm).toBe('blueprint');
    expect(opus?.run).toBe('blueprint-01');
    expect(opus?.name).toBe('Swatter Seesaw');
    expect(opus?.prediction).toBe('Swatter Seesaw does it.');
    expect(opus?.outcome.goal_met).toBe(true);
    expect(JSON.parse(opus?.sceneJson ?? '')).toEqual({ level: 'seesaw-launch', parts: [] });
    expect(opus?.summary).toEqual({ status: 'submitted', cost_usd: 0.24, duration_s: 79.1 });
    expect(entries[0]?.summary).toBeUndefined();
  });

  it('lines up one entry per model for a level and arm, in model order', () => {
    const entries = indexGallery(files, summaries);
    const six = lineup(entries, 'seesaw-launch', 'blueprint');
    expect(six.map((e) => e.model)).toEqual(['claude-sonnet-5-5', 'claude-opus-5-5']);
    expect(six.map((e) => e.run)).toEqual(['blueprint-01', 'blueprint-01']);
    expect(lineup(entries, 'seesaw-launch', 'blueprint', '02').map((e) => e.run)).toEqual(['blueprint-02']);
    expect(lineup(entries, 'bell-drop', 'blueprint')).toEqual([]);
  });
});

describe('on-demand files', () => {
  it('pick the loaders of one entry by path', () => {
    const load = () => Promise.resolve('');
    const files = {
      '../../gallery/bell-drop/claude-haiku-5-5/workshop-01/calls/0001-check.json': load,
      '../../gallery/bell-drop/claude-haiku-5-5/workshop-01/calls/0002-run.json': load,
      '../../gallery/bell-drop/claude-opus-5-5/workshop-01/calls/0001-run.json': load,
    };
    const mine = loadersFor(files, 'bell-drop/claude-haiku-5-5/workshop-01', (p) => callKey(p)?.key);
    expect(mine.map(([path]) => path.split('/').pop())).toEqual(['0001-check.json', '0002-run.json']);
    expect(transcriptKey('../../gallery/bell-drop/claude-haiku-5-5/workshop-01/transcript.jsonl')).toEqual({
      level: 'bell-drop',
      model: 'claude-haiku-5-5',
      run: 'workshop-01',
    });
  });
});

describe('the gallery page index', () => {
  const files = {
    '../../gallery/seesaw-launch/claude-sonnet-5-5/blueprint-01/submission.json': submission('seesaw-launch', 'Tilted plank', false),
    '../../gallery/seesaw-launch/claude-opus-5-5/blueprint-01/submission.json': submission('seesaw-launch', 'Swatter Seesaw', true),
    '../../gallery/bell-drop/claude-opus-5-5/workshop-01/submission.json': submission('bell-drop', 'Two planks', true),
  };
  const summaries = {
    '../../gallery/seesaw-launch/claude-opus-5-5/blueprint-01/summary.json': summary,
    '../../gallery/seesaw-launch/claude-haiku-5-5/workshop-01/summary.json': summary.replace('"status":"submitted"', '"status":"no_submission"').replace('claude-opus-5-5', 'claude-haiku-5-5'),
  };

  it('groups entries by level in model order', () => {
    const groups = byLevel(indexGallery(files, summaries));
    expect([...groups.keys()]).toEqual(['bell-drop', 'seesaw-launch']);
    expect(groups.get('seesaw-launch')?.map((e) => e.model)).toEqual(['claude-sonnet-5-5', 'claude-opus-5-5']);
  });

  it('lists the counted runs that never submitted', () => {
    expect(misses(indexGallery(files, summaries), summaries)).toEqual([
      { key: 'seesaw-launch/claude-haiku-5-5/workshop-01', level: 'seesaw-launch', model: 'claude-haiku-5-5', run: 'workshop-01', status: 'no_submission' },
    ]);
  });
});
