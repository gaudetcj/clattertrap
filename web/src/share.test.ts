import { describe, expect, it } from 'vitest';
import { decodeScene, encodeScene } from './share';

describe('a scene in a link', () => {
  const scene = { level: 'bell-drop', parts: [{ id: 'b1', kind: 'beam', x_cm: 120, y_cm: 210, length_cm: 120, angle_deg: -15 }] };

  it('round-trips through a URL-safe string with no padding', () => {
    const text = encodeScene(scene);
    expect(text).toMatch(/^[A-Za-z0-9_-]+$/);
    expect(decodeScene(text)).toEqual(scene);
  });

  it('is undefined for text that is not an encoded scene', () => {
    expect(decodeScene('not base64!')).toBeUndefined();
    expect(decodeScene(encodeScene({ level: 'x', parts: [] }).slice(0, 3))).toBeUndefined();
    expect(decodeScene(btoa('{"nope":1}').replace(/=/g, ''))).toBeUndefined();
  });
});
