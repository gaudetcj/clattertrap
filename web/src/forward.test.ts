import { describe, expect, it } from 'vitest';
import { forwardFor } from './forward';

describe('old landing links', () => {
  it('send a machine to the level page and a lineup to the arena', () => {
    expect(forwardFor('?entry=bell-drop/claude-opus-5-5/blueprint-01')).toBe('level.html?entry=bell-drop/claude-opus-5-5/blueprint-01');
    expect(forwardFor('?level=seesaw-launch&arm=workshop')).toBe('arena.html?level=seesaw-launch&arm=workshop');
    expect(forwardFor('?arm=blueprint')).toBe('arena.html?arm=blueprint');
  });

  it('leave a plain visit on the home page', () => {
    expect(forwardFor('')).toBeUndefined();
    expect(forwardFor('?utm=x')).toBeUndefined();
  });
});
