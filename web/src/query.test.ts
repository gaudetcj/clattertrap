import { describe, expect, it } from 'vitest';
import { param, search } from './query';

describe('query strings', () => {
  it('read one parameter, empty counting as absent', () => {
    expect(param('?entry=seesaw-launch%2Fclaude-opus-5-5%2Fblueprint-01', 'entry')).toBe(
      'seesaw-launch/claude-opus-5-5/blueprint-01',
    );
    expect(param('?level=bell-drop&arm=workshop', 'arm')).toBe('workshop');
    expect(param('?entry=', 'entry')).toBeUndefined();
    expect(param('', 'entry')).toBeUndefined();
  });

  it('write parameters in the given order, encoded', () => {
    expect(search({ entry: 'seesaw-launch/claude-opus-5-5/blueprint-01' })).toBe(
      '?entry=seesaw-launch%2Fclaude-opus-5-5%2Fblueprint-01',
    );
    expect(search({ level: 'bell-drop', arm: 'workshop' })).toBe('?level=bell-drop&arm=workshop');
    expect(search({})).toBe('');
  });
});
