import { describe, expect, it } from 'vitest';
import { Camera } from './camera';

describe('Camera.fit', () => {
  it('maps a 4 by 3 m world onto an 800 by 600 canvas with y up', () => {
    const c = Camera.fit(400, 300, 800, 600);
    expect(c.scale_px_per_m).toBe(200);
    expect(c.toPx(0, 0)).toEqual({ x: 0, y: 600 });
    expect(c.toPx(4, 3)).toEqual({ x: 800, y: 0 });
    expect(c.toPx(1, 1)).toEqual({ x: 200, y: 400 });
    expect(c.px(0.06)).toBe(12);
  });

  it('letterboxes a wider canvas', () => {
    const c = Camera.fit(400, 300, 1000, 600);
    expect(c.scale_px_per_m).toBe(200);
    expect(c.toPx(0, 0)).toEqual({ x: 100, y: 600 });
  });

  it('letterboxes a taller canvas', () => {
    const c = Camera.fit(400, 300, 800, 800);
    expect(c.toPx(0, 0)).toEqual({ x: 0, y: 700 });
    expect(c.toPx(0, 3)).toEqual({ x: 0, y: 100 });
  });

  it('rounds the origin to whole pixels', () => {
    const c = Camera.fit(400, 300, 801, 601);
    const o = c.toPx(0, 0);
    expect(Number.isInteger(o.x) && Number.isInteger(o.y)).toBe(true);
  });
});

describe('Camera.toWorld', () => {
  it('inverts toPx, so a pixel maps back to the metres it came from', () => {
    const camera = Camera.fit(400, 300, 400, 300);
    const px = camera.toPx(1.25, 0.5);
    expect(camera.toWorld(px.x, px.y)).toEqual({ x_m: 1.25, y_m: 0.5 });
    // The bottom-left corner of the world is the bottom-left pixel.
    expect(camera.toWorld(0, 300)).toEqual({ x_m: 0, y_m: 0 });
  });
});
