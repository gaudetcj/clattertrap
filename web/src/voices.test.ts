import { describe, expect, it } from 'vitest';
import type { ReplayEvent, Setup } from './decode';
import { voicesAt } from './voices';

const setup: Setup = {
  width_cm: 400,
  height_cm: 300,
  time_limit_s: 20,
  zones: [],
  goal: { kind: 'ring', target: 'bell', via: [] },
  names: ['bell', 'ball', 'beam', 'pivot'],
  slots: [
    { part: 0, visual: { shape: 'circle', r_m: 0.1, look: 'brass' } },
    { part: 1, visual: { shape: 'circle', r_m: 0.05, look: 'steel' } },
    { part: 2, visual: { shape: 'box', w_m: 1, h_m: 0.04, look: 'wood' } },
    { part: 3, visual: { shape: 'pivot' } },
  ],
};

const speeds: Record<number, number> = { 0: 0, 1: 1, 2: 3, 3: 0 };
const replay = { setup, speed: (part: number) => speeds[part] ?? 0 };

function contact(a: number, b: number): ReplayEvent {
  return { kind: 'contact', a, b };
}

describe('voicesAt', () => {
  it('voices a contact by the two looks and the faster part', () => {
    expect(voicesAt(replay, [contact(1, 2)], 6)).toEqual([{ timbre: 'ping', gain: 1 }]);
    expect(voicesAt(replay, [contact(0, 1)], 6)).toEqual([{ timbre: 'tick', gain: 0.5 }]);
  });

  it('voices the goal and nothing for rest, waypoints, or parts without a look', () => {
    expect(voicesAt(replay, [{ kind: 'goal_met', a: 0, b: 0 }], 6)).toEqual([{ timbre: 'bell', gain: 1 }]);
    expect(voicesAt(replay, [{ kind: 'at_rest', a: 1, b: 0 }, { kind: 'via', a: 2, b: 0 }, contact(3, 1)], 6)).toEqual([]);
  });

  it('plays at most the cap', () => {
    const events = [contact(1, 2), contact(0, 1), contact(0, 2)];
    expect(voicesAt(replay, events, 2)).toHaveLength(2);
    expect(voicesAt(replay, events, 0)).toEqual([]);
  });
});
