import { describe, expect, it } from 'vitest';
import type { Entry } from './gallery';
import { goalLine, modelsLine } from './models_here';

function entry(model: string, level: string, arm: string, goal_met: boolean, time_s: number): Entry {
  return {
    key: `${level}/${model}/${arm}-01`,
    level,
    model,
    arm,
    run: `${arm}-01`,
    name: 'm',
    prediction: '',
    sceneJson: '{}',
    outcome: { goal_met, end: goal_met ? 'goal_met' : 'time_limit', time_s, steps: 1, parts_used: 1, chain_length: 1, event_hash: '0', state_hash: '0', rules_version: 1 },
  };
}

describe('how the models did here', () => {
  const entries = [
    entry('claude-opus-5-5', 'bell-drop', 'blueprint', true, 0.7),
    entry('x-ai-grok-4.3', 'bell-drop', 'blueprint', true, 1.2),
    entry('claude-haiku-4-5', 'bell-drop', 'blueprint', false, 20),
    entry('claude-opus-5-5', 'bell-drop', 'workshop', true, 0.5),
    entry('claude-opus-5-5', 'seesaw-launch', 'blueprint', true, 0.9),
  ];

  it('counts the blueprint solves and names the fastest', () => {
    expect(modelsLine(entries, 'bell-drop', 'ring')).toBe('2 of 3 models rang it blind; the fastest was OPUS 5.5 in 0.7 s.');
  });

  it('says so when none did, and when none played', () => {
    expect(modelsLine([entry('claude-haiku-4-5', 'bell-drop', 'blueprint', false, 20)], 'bell-drop', 'press')).toBe('0 of 1 models pressed it blind.');
    expect(modelsLine(entries, 'nope', 'fill')).toBe('No model has played this level yet.');
  });
});

describe('the goal in words', () => {
  it('names the verb, the target, a count, and the waypoints in order', () => {
    expect(goalLine({ kind: 'ring', target: 'bell', via: [] })).toBe('Ring the bell.');
    expect(goalLine({ kind: 'fill', target: 'basket', via: [], count: 3 })).toBe('Fill the basket with 3 things.');
    expect(goalLine({ kind: 'press', target: 'button', via: ['post', 'beam'] })).toBe('Press the button, touching post then beam on the way.');
    expect(goalLine({ kind: 'pop', target: 'balloon', via: ['pad'] })).toBe('Pop the balloon, touching pad on the way.');
  });
});
