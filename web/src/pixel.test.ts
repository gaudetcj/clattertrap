import { describe, expect, it } from 'vitest';
import { PALETTE, paletteLut, pixelGrid, rgbPalette, snapToPalette } from './pixel';

describe('pixelGrid', () => {
  it('picks a whole-number upscale so one pixel is about the asked centimetres', () => {
    // 400 cm at 1 cm per pixel wants 400 low-res pixels across 860 css pixels: upscale 2.
    expect(pixelGrid(860, 645, 400, 1)).toEqual({ scale: 2, w: 430, h: 322 });
    // 2 cm per pixel wants 200 across 280: upscale 1, the panel's own size.
    expect(pixelGrid(280, 210, 400, 2)).toEqual({ scale: 1, w: 280, h: 210 });
    // 1.5 cm per pixel wants 267 across 860: upscale 3.
    expect(pixelGrid(860, 645, 400, 1.5)).toEqual({ scale: 3, w: 286, h: 215 });
  });

  it('never upscales by less than one', () => {
    expect(pixelGrid(100, 75, 400, 0.1).scale).toBe(1);
  });
});

describe('snapToPalette', () => {
  const palette = rgbPalette(['#ff0000', '#000000', '#f6f1e7']);
  const lut = paletteLut(palette);

  it('builds one table entry per five-bit colour', () => {
    expect(lut.length).toBe(32768);
  });

  it('leaves palette colours alone and moves blends to the nearest one', () => {
    const data = new Uint8ClampedArray([255, 0, 0, 255, 250, 12, 4, 255, 120, 110, 100, 255]);
    snapToPalette(data, palette, lut);
    expect(Array.from(data)).toEqual([255, 0, 0, 255, 255, 0, 0, 255, 0, 0, 0, 255]);
  });

  it('keeps alpha and snaps a pixel that is not at the start of its buffer', () => {
    const buffer = new ArrayBuffer(16);
    const data = new Uint8ClampedArray(buffer, 4, 8);
    data.set([240, 230, 220, 128, 10, 0, 0, 7]);
    snapToPalette(data, palette, lut);
    expect(Array.from(data)).toEqual([246, 241, 231, 128, 0, 0, 0, 7]);
    expect(Array.from(new Uint8ClampedArray(buffer, 0, 4))).toEqual([0, 0, 0, 0]);
  });

  it('ships a palette of a few dozen colours that includes the paper and the stamp greens and reds', () => {
    expect(PALETTE.length).toBeGreaterThan(16);
    expect(PALETTE.length).toBeLessThan(48);
    expect(PALETTE).toContain('#1f7a3a');
    expect(PALETTE).toContain('#b3261e');
  });
});
