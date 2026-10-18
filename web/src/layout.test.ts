import { describe, expect, it } from 'vitest';
import {
  COLS,
  FOOTER_H,
  FOOTER_LINES,
  FOOTER_CHARS,
  footerRect,
  gridScale,
  gridSize,
  HEADER_H,
  headerRect,
  panelRects,
  panelSize,
  worldRect,
} from './layout';
import { TRAY_H } from './tray_view';

describe('panel layout', () => {
  const panel = panelSize(400, 300);

  it('stacks a header, the world, the tray, and a footer', () => {
    expect([HEADER_H, TRAY_H, FOOTER_H]).toEqual([40, 20, 44]);
    expect(panel).toEqual({ w: 400, h: 404 });
    expect(FOOTER_LINES).toBe(5);
    expect(FOOTER_CHARS).toBe(66);
  });

  it('lays six panels three across with even grid dimensions', () => {
    const rects = panelRects(6, COLS, panel);
    expect(rects).toHaveLength(6);
    expect(rects[0]).toEqual({ x: 0, y: 0, w: 400, h: 404 });
    expect(rects[2]).toEqual({ x: 800, y: 0, w: 400, h: 404 });
    expect(rects[3]).toEqual({ x: 0, y: 404, w: 400, h: 404 });
    expect(rects[5]).toEqual({ x: 800, y: 404, w: 400, h: 404 });
    const grid = gridSize(6, COLS, panel);
    expect(grid).toEqual({ w: 1200, h: 808 });
    expect(grid.w % 2).toBe(0);
    expect(grid.h % 2).toBe(0);
  });

  it('lays one panel alone and four in two rows', () => {
    expect(gridSize(1, COLS, panel)).toEqual({ w: 400, h: 404 });
    expect(gridSize(4, COLS, panel)).toEqual({ w: 1200, h: 808 });
    expect(panelRects(4, COLS, panel)[3]).toEqual({ x: 0, y: 404, w: 400, h: 404 });
    expect(panelRects(0, COLS, panel)).toEqual([]);
  });

  it('places the world between the header and the footer', () => {
    const rect = { x: 800, y: 404, w: 400, h: 404 };
    expect(headerRect(rect)).toEqual({ x: 800, y: 404, w: 400, h: 40 });
    expect(worldRect(rect)).toEqual({ x: 800, y: 444, w: 400, h: 320 });
    expect(footerRect(rect)).toEqual({ x: 800, y: 764, w: 400, h: 44 });
  });

  it('upscales by a whole number that fits the page', () => {
    expect(gridScale(1280, 1200)).toBe(1);
    expect(gridScale(2560, 1200)).toBe(2);
    expect(gridScale(800, 1200)).toBe(1);
    expect(gridScale(860, 400)).toBe(2);
  });
});
