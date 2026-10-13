import { describe, expect, it } from 'vitest';
import { decodeEvents, decodeFrame, decodeOutcome, decodeSetup } from './decode';

const SETUP = JSON.stringify({
  width_cm: 400,
  height_cm: 300,
  time_limit_s: 20,
  zones: [{ x_cm: 30, y_cm: 0, width_cm: 310, height_cm: 300 }],
  goal: { kind: 'ring', target: 'bell', via: [] },
  names: ['ball0', 'bell', 'b1', 'floor'],
  slots: [
    { part: 3, visual: { shape: 'box', w_m: 4.5, h_m: 1, look: 'wood' } },
    { part: 0, visual: { shape: 'circle', r_m: 0.06, look: 'steel' } },
    { part: 1, visual: { shape: 'circle', r_m: 0.1, look: 'brass' } },
    { part: 1, visual: { shape: 'pivot' } },
    { part: 2, visual: { shape: 'trough', inner_w_m: 0.3, inner_h_m: 0.2, wall_m: 0.02, look: 'wicker' } },
  ],
});

describe('decodeSetup', () => {
  it('reads the world, goal, names, and slots', () => {
    const s = decodeSetup(SETUP);
    expect(s.width_cm).toBe(400);
    expect(s.goal.kind).toBe('ring');
    expect(s.names[1]).toBe('bell');
    expect(s.slots).toHaveLength(5);
    expect(s.slots[3]?.visual).toEqual({ shape: 'pivot' });
    const trough = s.slots[4]?.visual;
    expect(trough?.shape === 'trough' && trough.look === 'wicker').toBe(true);
  });

  it('keeps a fill goal count', () => {
    const s = decodeSetup(
      SETUP.replace('"kind":"ring","target":"bell"', '"kind":"fill","target":"basket","count":2'),
    );
    expect(s.goal.kind).toBe('fill');
    expect(s.goal.count).toBe(2);
  });

  it('rejects an unknown look or shape', () => {
    expect(() => decodeSetup(SETUP.replace('"wicker"', '"marble"'))).toThrow(/look/);
    expect(() => decodeSetup(SETUP.replace('"shape":"pivot"', '"shape":"blob"'))).toThrow(/shape/);
    expect(() => decodeSetup('{}')).toThrow();
  });
});

describe('decodeEvents', () => {
  it('turns kind triples into named events', () => {
    const events = decodeEvents(new Uint16Array([0, 2, 5, 2, 0, 0, 3, 7, 0, 4, 1, 0, 1, 6, 0]));
    expect(events).toEqual([
      { kind: 'contact', a: 2, b: 5 },
      { kind: 'goal_met', a: 0, b: 0 },
      { kind: 'at_rest', a: 7, b: 0 },
      { kind: 'left_world', a: 1, b: 0 },
      { kind: 'via', a: 6, b: 0 },
    ]);
  });
});

describe('decodeFrame', () => {
  it('reads four numbers per slot and marks removed bodies', () => {
    const poses = decodeFrame(new Float32Array([1, 2, 0.5, 1, 0, 0, 0, 0]));
    expect(poses).toHaveLength(2);
    expect(poses[0]).toEqual({ x_m: 1, y_m: 2, angle_rad: 0.5, alive: true });
    expect(poses[1]?.alive).toBe(false);
  });
});

describe('decodeOutcome', () => {
  it('reads the hashes as strings and the end reason', () => {
    const o = decodeOutcome(
      '{"goal_met":true,"end":"goal_met","time_s":1.925,"steps":231,"parts_used":2,"chain_length":2,"event_hash":"41a3e9905cb8f870","state_hash":"bbf5aaffac928e50","rules_version":1}',
    );
    expect(o.end).toBe('goal_met');
    expect(o.state_hash).toBe('bbf5aaffac928e50');
    expect(o.steps).toBe(231);
  });

  it('rejects an unknown end', () => {
    expect(() => decodeOutcome('{"goal_met":false,"end":"gave_up","time_s":1,"steps":1,"parts_used":0,"chain_length":0,"event_hash":"0","state_hash":"0","rules_version":1}')).toThrow(/end/);
  });
});
