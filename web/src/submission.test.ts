import { describe, expect, it } from 'vitest';
import { parseOpened } from './submission';

const SCENE = { level: 'bell-drop', parts: [{ id: 'b1', kind: 'beam', x_cm: 120, y_cm: 210, length_cm: 120, angle_deg: -15 }] };

describe('parseOpened', () => {
  it('takes a bare scene as it is', () => {
    const text = JSON.stringify(SCENE);
    const opened = parseOpened(text);
    expect(opened.levelId).toBe('bell-drop');
    expect(opened.sceneJson).toBe(text);
    expect(opened.name).toBeUndefined();
    expect(opened.prediction).toBeUndefined();
  });

  it('takes a submission record and keeps its name and prediction', () => {
    const record = {
      level: 'bell-drop',
      arm: 'workshop',
      name: 'Two planks',
      prediction: 'The ball rolls down both beams and rings the bell.',
      scene: SCENE,
      outcome: { goal_met: true },
    };
    const opened = parseOpened(JSON.stringify(record));
    expect(opened.levelId).toBe('bell-drop');
    expect(JSON.parse(opened.sceneJson)).toEqual(SCENE);
    expect(opened.name).toBe('Two planks');
    expect(opened.prediction).toBe('The ball rolls down both beams and rings the bell.');
  });

  it('refuses text that is not JSON or not a scene', () => {
    expect(() => parseOpened('{ not json')).toThrow(/JSON/);
    expect(() => parseOpened('{"level": "bell-drop"}')).toThrow(/scene or a submission/);
    expect(() => parseOpened('[1, 2]')).toThrow(/scene or a submission/);
    expect(() => parseOpened('{"scene": {"parts": []}}')).toThrow(/scene or a submission/);
  });
});
