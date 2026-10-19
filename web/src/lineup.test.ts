import { describe, expect, it } from 'vitest';
import { defaultLineup, lanesFor, lineupModels, lineupNames, pickFrom } from './arena_lanes';
import type { Entry } from './gallery';

function entry(model: string, level: string, arm: string): Entry {
  return {
    key: `${level}/${model}/${arm}-01`,
    level,
    model,
    arm,
    run: `${arm}-01`,
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

// Leaderboard order, strongest first.
const ranked = [
  'claude-opus-5-5',
  'openai-gpt-5.6-luna',
  'claude-sonnet-5-5',
  'deepseek-deepseek-v4.1-flash',
  'x-ai-grok-4.3',
  'claude-haiku-5-5',
  'z-ai-glm-5.3-flash',
  'claude-opus-4-5',
  'qwen-qwen3.6-35b-a3b',
  'xiaomi-mimo-v2.6-flash',
  'claude-sonnet-4-5',
  'claude-haiku-4-5',
];

describe('the pick', () => {
  it('reads a named lineup or a custom model list, ignoring unknown names', () => {
    expect(pickFrom('?lineup=claude').lineup).toBe('claude');
    expect(pickFrom('?lineup=families').lineup).toBe('families');
    expect(pickFrom('?lineup=nope').lineup).toBeUndefined();
    expect(pickFrom('?models=claude-opus-5-5,x-ai-grok-4.3').models).toEqual(['claude-opus-5-5', 'x-ai-grok-4.3']);
    expect(pickFrom('?models=').models).toBeUndefined();
  });
});

describe('lineups', () => {
  it('take one model per family, the strongest, in leaderboard order', () => {
    expect(lineupModels('families', ranked, ranked)).toEqual([
      'claude-opus-5-5',
      'openai-gpt-5.6-luna',
      'deepseek-deepseek-v4.1-flash',
      'x-ai-grok-4.3',
      'z-ai-glm-5.3-flash',
      'qwen-qwen3.6-35b-a3b',
      'xiaomi-mimo-v2.6-flash',
    ]);
  });

  it('keep only models present on the level and arm', () => {
    const present = ['claude-haiku-4-5', 'x-ai-grok-4.3', 'claude-haiku-5-5'];
    expect(lineupModels('all', ranked, present)).toEqual(['x-ai-grok-4.3', 'claude-haiku-5-5', 'claude-haiku-4-5']);
    expect(lineupModels('families', ranked, present)).toEqual(['x-ai-grok-4.3', 'claude-haiku-5-5']);
    expect(lineupModels('claude', ranked, present)).toEqual(['claude-haiku-5-5', 'claude-haiku-4-5']);
    expect(lineupModels('open', ranked, ranked)).toEqual([
      'deepseek-deepseek-v4.1-flash',
      'z-ai-glm-5.3-flash',
      'qwen-qwen3.6-35b-a3b',
      'xiaomi-mimo-v2.6-flash',
    ]);
  });

  it('default to one per family once a level has more than six models', () => {
    expect(defaultLineup(6)).toBe('all');
    expect(defaultLineup(7)).toBe('families');
  });

  it('play a custom model list in the order given, skipping models without an entry', () => {
    const levels = [{ id: 'bell-drop', level: '{"id":"bell-drop"}', reference: '{}' }];
    const entries = [entry('claude-opus-5-5', 'bell-drop', 'blueprint'), entry('claude-haiku-4-5', 'bell-drop', 'blueprint')];
    const pick = { level: 'bell-drop', arm: 'blueprint', models: ['claude-haiku-4-5', 'x-ai-grok-4.3', 'claude-opus-5-5'] };
    expect(lanesFor(entries, levels, pick, ranked).map((l) => l.chrome.label)).toEqual(['HAIKU 4.5', 'OPUS 5.5']);
    // Without a list, the lanes follow the leaderboard.
    expect(lanesFor(entries, levels, { level: 'bell-drop', arm: 'blueprint' }, ranked).map((l) => l.chrome.label)).toEqual(['OPUS 5.5', 'HAIKU 4.5']);
  });
});

describe('lineup names', () => {
  it('say how many of the models present each lineup plays', () => {
    const present = ['claude-opus-5-5', 'claude-haiku-5-5', 'x-ai-grok-4.3', 'qwen-qwen3.6-35b-a3b'];
    expect(lineupNames(ranked, present)).toEqual({
      families: 'one per family (3)',
      all: 'all models (4)',
      claude: 'Claude (2)',
      open: 'open weights (1)',
    });
  });
});
