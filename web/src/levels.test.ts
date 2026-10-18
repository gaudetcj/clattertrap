import { describe, expect, it } from 'vitest';
import { goalKindOf, indexFixtures, rulesVersionOf, titleOf } from './levels';

const level = JSON.stringify({ id: 'bell-drop', title: 'Ring the bell', rules_version: 1, goal: { kind: 'ring', target: 'bell' } });

describe('levels', () => {
  it('pairs each level with its reference and sorts by id', () => {
    const fixtures = indexFixtures({
      '../../levels/seesaw-launch/level.json': '{"id":"seesaw-launch"}',
      '../../levels/seesaw-launch/reference.json': '{"parts":[]}',
      '../../levels/bell-drop/level.json': level,
      '../../levels/bell-drop/reference.json': '{"parts":[1]}',
      '../../levels/half/level.json': '{}',
    });
    expect(fixtures.map((f) => f.id)).toEqual(['bell-drop', 'seesaw-launch']);
    expect(fixtures[0]?.reference).toBe('{"parts":[1]}');
  });

  it('reads the rules version, the title, and the goal kind', () => {
    expect(rulesVersionOf(level)).toBe(1);
    expect(titleOf(level)).toBe('Ring the bell');
    expect(goalKindOf(level)).toBe('ring');
    expect(rulesVersionOf('{}')).toBe(-1);
    expect(titleOf('[]')).toBe('');
    expect(goalKindOf('{"goal":{"kind":"sing"}}')).toBeUndefined();
  });
});
