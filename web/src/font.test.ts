import { describe, expect, it } from 'vitest';
import { drawText, GLYPH_H, GLYPH_W, hasGlyph, textWidth } from './font';

class Rects {
  rects: [number, number, number, number][] = [];
  fillStyle = '';
  fillRect(x: number, y: number, w: number, h: number): void {
    this.rects.push([x, y, w, h]);
  }
}

describe('the pixel font', () => {
  it('has a glyph for every character the stamps, the clock, and the tray use', () => {
    for (const ch of 'RANG PRESSED FILLED POPPED MISSED CAME TO REST 0123456789.s/:') {
      expect(hasGlyph(ch), ch).toBe(true);
    }
  });

  it('is five by seven with a one-pixel gap', () => {
    expect([GLYPH_W, GLYPH_H]).toEqual([5, 7]);
    expect(textWidth('RANG', 1)).toBe(23);
    expect(textWidth('RANG', 3)).toBe(69);
    expect(textWidth('', 2)).toBe(0);
  });

  it('fills one rectangle per lit pixel, scaled, and skips blanks', () => {
    const ctx = new Rects();
    drawText(ctx, 'I', 10, 20, 2, '#fff');
    // I: a bar of three on top and bottom, a stem of one for five rows.
    expect(ctx.rects.length).toBe(11);
    expect(ctx.rects[0]).toEqual([12, 20, 2, 2]);
    expect(ctx.fillStyle).toBe('#fff');
    const blank = new Rects();
    drawText(blank, ' ~', 0, 0, 1, '#fff');
    expect(blank.rects.length).toBe(0);
  });
});
