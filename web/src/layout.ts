// Where panels sit on the low-res grid: each one is a header, the world, the inventory tray, and a
// footer for the prediction, laid out in rows with no gutter.

import { GLYPH_H } from './font';
import { TRAY_H } from './tray_view';

export interface Size {
  w: number;
  h: number;
}

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

// Panels per row.
export const COLS = 3;
// The model label and the machine name, drawn at twice the font size.
export const HEADER_H = 40;
// Prediction lines in the footer, at the font size, and the characters that fit in one.
export const FOOTER_LINES = 5;
export const FOOTER_CHARS = 66;
const LINE_H = GLYPH_H + 1;
export const FOOTER_H = 4 + FOOTER_LINES * LINE_H;

// A panel for a world of the given size at one pixel per centimetre.
export function panelSize(world_w: number, world_h: number): Size {
  return { w: world_w, h: HEADER_H + world_h + TRAY_H + FOOTER_H };
}

// The rects of `n` panels in rows of `cols`, row-major from the top left.
export function panelRects(n: number, cols: number, panel: Size): Rect[] {
  const rects: Rect[] = [];
  for (let i = 0; i < n; i += 1) {
    rects.push({ x: (i % cols) * panel.w, y: Math.floor(i / cols) * panel.h, w: panel.w, h: panel.h });
  }
  return rects;
}

// The grid that holds `n` panels.
export function gridSize(n: number, cols: number, panel: Size): Size {
  const across = Math.min(n, cols);
  return { w: across * panel.w, h: Math.ceil(n / cols) * panel.h };
}

export function headerRect(panel: Rect): Rect {
  return { x: panel.x, y: panel.y, w: panel.w, h: HEADER_H };
}

// The world and its tray.
export function worldRect(panel: Rect): Rect {
  return { x: panel.x, y: panel.y + HEADER_H, w: panel.w, h: panel.h - HEADER_H - FOOTER_H };
}

export function footerRect(panel: Rect): Rect {
  return { x: panel.x, y: panel.y + panel.h - FOOTER_H, w: panel.w, h: FOOTER_H };
}

// The whole-number upscale that fits a grid into a page width, never below one.
export function gridScale(css_w: number, grid_w: number): number {
  return Math.max(1, Math.floor(css_w / grid_w));
}
