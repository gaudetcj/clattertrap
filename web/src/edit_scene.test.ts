import { describe, expect, it } from 'vitest';
import { addPart, defaultPart, fieldsOf, moveTo, nextId, parseScene, removePart, sceneJson, setField, type EditScene, type Ranges } from './edit_scene';

const ranges: Ranges = {
  beam_length: { min_m: 0.2, max_m: 1.5 },
  seesaw_length: { min_m: 0.4, max_m: 1.5 },
  crate_side: { min_m: 0.1, max_m: 0.4 },
  pendulum_length: { min_m: 0.2, max_m: 1.2 },
  bucket_size: { min_m: 0.15, max_m: 0.6 },
  funnel_mouth: { min_m: 0.3, max_m: 1.5 },
  funnel_height: { min_m: 0.2, max_m: 0.8 },
  funnel_throat_min_m: 0.14,
  rope_length: { min_m: 0.1, max_m: 2 },
};
const world = { width_cm: 400, height_cm: 300 };

describe('an edited scene', () => {
  const empty: EditScene = { level: 'bell-drop', parts: [] };

  it('names a new part after its kind, skipping ids in use', () => {
    expect(nextId(empty, 'beam')).toBe('b1');
    const one = addPart(empty, defaultPart('beam', 'b1', 100, 200, ranges));
    expect(nextId(one, 'beam')).toBe('b2');
    expect(nextId(one, 'ball')).toBe('k1');
    expect(nextId(one, 'spring_pad')).toBe('sp1');
  });

  it('gives each kind its fields with defaults inside the rules', () => {
    expect(defaultPart('beam', 'b1', 100, 200, ranges)).toEqual({ id: 'b1', kind: 'beam', x_cm: 100, y_cm: 200, length_cm: 100, angle_deg: 0 });
    expect(defaultPart('ball', 'k1', 50, 60, ranges)).toEqual({ id: 'k1', kind: 'ball', x_cm: 50, y_cm: 60, material: 'steel' });
    expect(defaultPart('funnel', 'f1', 50, 60, ranges)).toEqual({ id: 'f1', kind: 'funnel', x_cm: 50, y_cm: 60, mouth_cm: 80, throat_cm: 20, height_cm: 40 });
    expect(defaultPart('pin', 'n1', 50, 60, ranges)).toEqual({ id: 'n1', kind: 'pin', x_cm: 50, y_cm: 60, angle_deg: -90 });
    expect(defaultPart('rope', 'r1', 0, 0, ranges)).toEqual({ id: 'r1', kind: 'rope', a: '', b: '', max_length_cm: 100 });
  });

  it('lists the numeric fields of a kind with the range the rules give them', () => {
    const beam = fieldsOf('beam', ranges, world);
    expect(beam.map((f) => f.name)).toEqual(['x_cm', 'y_cm', 'length_cm', 'angle_deg']);
    expect(beam.find((f) => f.name === 'length_cm')).toEqual({ name: 'length_cm', min: 20, max: 150, step: 1 });
    expect(beam.find((f) => f.name === 'x_cm')).toEqual({ name: 'x_cm', min: 0, max: 400, step: 1 });
    expect(beam.find((f) => f.name === 'angle_deg')).toEqual({ name: 'angle_deg', min: -180, max: 180, step: 1 });
    expect(fieldsOf('ball', ranges, world).map((f) => f.name)).toEqual(['x_cm', 'y_cm']);
    expect(fieldsOf('rope', ranges, world)).toEqual([{ name: 'max_length_cm', min: 10, max: 200, step: 1 }]);
    expect(fieldsOf('funnel', ranges, world).find((f) => f.name === 'throat_cm')).toEqual({ name: 'throat_cm', min: 14, max: 150, step: 1 });
  });

  it('moves, edits, and removes parts without touching the others', () => {
    let scene = addPart(empty, defaultPart('beam', 'b1', 100, 200, ranges));
    scene = addPart(scene, defaultPart('ball', 'k1', 50, 60, ranges));
    scene = addPart(scene, { ...defaultPart('rope', 'r1', 0, 0, ranges), a: 'b1', b: 'k1' });
    const moved = moveTo(scene, 'k1', 70.4, 80.6);
    expect(moved.parts[1]).toEqual({ id: 'k1', kind: 'ball', x_cm: 70, y_cm: 81, material: 'steel' });
    expect(moved.parts[0]).toBe(scene.parts[0]);
    const turned = setField(moved, 'b1', 'angle_deg', -15);
    expect(turned.parts[0]).toEqual({ id: 'b1', kind: 'beam', x_cm: 100, y_cm: 200, length_cm: 100, angle_deg: -15 });
    // Removing a part also removes the ropes tied to it.
    const fewer = removePart(turned, 'k1');
    expect(fewer.parts.map((p) => p.id)).toEqual(['b1']);
  });

  it('reads and writes scene JSON, refusing anything else', () => {
    const scene = addPart(empty, defaultPart('beam', 'b1', 100, 200, ranges));
    const json = sceneJson(scene);
    expect(json).toBe('{"level":"bell-drop","parts":[{"id":"b1","kind":"beam","x_cm":100,"y_cm":200,"length_cm":100,"angle_deg":0}]}');
    expect(parseScene(json)).toEqual(scene);
    expect(() => parseScene('{"level":"bell-drop"}')).toThrow();
    expect(() => parseScene('{"level":"bell-drop","parts":[{"kind":"beam"}]}')).toThrow();
  });
});
