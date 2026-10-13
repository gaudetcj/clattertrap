import { describe, expect, it } from 'vitest';
import type { Look, ReplayEvent } from './decode';
import { voiceFor } from './sound';

const looks: Record<number, Look> = { 0: 'wood', 1: 'steel', 2: 'rubber', 3: 'brass', 4: 'pin', 5: 'balloon', 6: 'domino' };
const lookOf = (part: number): Look | null => looks[part] ?? null;

function contact(a: number, b: number): ReplayEvent {
  return { kind: 'contact', a, b };
}

describe('voiceFor', () => {
  it('picks a timbre from the harder of the two looks', () => {
    expect(voiceFor(contact(0, 6), lookOf, 1, 'ring')?.timbre).toBe('knock');
    expect(voiceFor(contact(0, 1), lookOf, 1, 'ring')?.timbre).toBe('ping');
    expect(voiceFor(contact(2, 0), lookOf, 1, 'ring')?.timbre).toBe('thud');
    expect(voiceFor(contact(0, 3), lookOf, 1, 'ring')?.timbre).toBe('tick');
  });

  it('pops when a pin meets a balloon, at full volume', () => {
    expect(voiceFor(contact(4, 5), lookOf, 0.1, 'pop')).toEqual({ timbre: 'pop', gain: 1 });
    expect(voiceFor(contact(5, 4), lookOf, 0.1, 'pop')).toEqual({ timbre: 'pop', gain: 1 });
  });

  it('scales contact volume with speed inside a floor and a ceiling', () => {
    expect(voiceFor(contact(0, 6), lookOf, 0, 'ring')?.gain).toBeCloseTo(0.15);
    expect(voiceFor(contact(0, 6), lookOf, 1, 'ring')?.gain).toBeCloseTo(0.5);
    expect(voiceFor(contact(0, 6), lookOf, 9, 'ring')?.gain).toBe(1);
  });

  it('voices the goal by its kind and stays quiet for the rest', () => {
    const met: ReplayEvent = { kind: 'goal_met', a: 0, b: 0 };
    expect(voiceFor(met, lookOf, 0, 'ring')?.timbre).toBe('bell');
    expect(voiceFor(met, lookOf, 0, 'press')?.timbre).toBe('click');
    expect(voiceFor(met, lookOf, 0, 'fill')?.timbre).toBe('thump');
    expect(voiceFor(met, lookOf, 0, 'pop')).toBeNull();
    expect(voiceFor({ kind: 'at_rest', a: 0, b: 0 }, lookOf, 0, 'ring')).toBeNull();
    expect(voiceFor({ kind: 'left_world', a: 0, b: 0 }, lookOf, 0, 'ring')).toBeNull();
    expect(voiceFor({ kind: 'via', a: 0, b: 0 }, lookOf, 0, 'ring')).toBeNull();
  });

  it('stays silent for a part it cannot see', () => {
    expect(voiceFor(contact(0, 99), lookOf, 1, 'ring')).toBeNull();
  });
});
