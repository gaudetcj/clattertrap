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
  it('has a glyph for every character the stamps, the clock, the tray, the labels, and predictions use', () => {
    const text =
      'RANG PRESSED FILLED POPPED MISSED CAME TO REST REPLAY MISMATCH ATTEMPT OF SUBMISSION 0123456789.s/:' +
      'abcdefghijklmnopqrstuvwxyz' +
      'ABCDEFGHIJKLMNOPQRSTUVWXYZ' +
      "',-()?!%+=;°\"" +
      "□";
    for (const ch of text) {
      expect(hasGlyph(ch), ch).toBe(true);
    }
    expect(hasGlyph('~')).toBe(false);
  });

  it('draws every glyph as seven rows of five bits', () => {
    for (const ch of 'abcdefghijklmnopqrstuvwxyz,-()?!%+=;°') {
      const ctx = new Rects();
      drawText(ctx, ch, 0, 0, 1, '#fff');
      expect(ctx.rects.length, ch).toBeGreaterThan(0);
      for (const [x, y] of ctx.rects) {
        expect(x, ch).toBeLessThan(5);
        expect(y, ch).toBeLessThan(7);
      }
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
