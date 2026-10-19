// Owns the grid canvas: one low-res backing store upscaled by a whole number, panels laid out by
// `layout`, each with its chrome (model, machine name, prediction) painted once and its world
// repainted and palette-snapped only when it moved.

import type { Setup } from './decode';
import { drawText, GLYPH_H } from './font';
import {
  bestColumns,
  COLS,
  FOOTER_CHARS,
  FOOTER_LINES,
  footerRect,
  fitScale,
  gridSize,
  headerRect,
  panelRects,
  panelSize,
  worldRect,
  type Rect,
  type Size,
} from './layout';
import type { Link } from './links';
import { PanelView, PAPER, type PanelCtx, type PanelFrame } from './panel_view';
import { PALETTE, paletteLut, rgbPalette, snapToPalette } from './pixel';
import { legible, wrapText } from './text';
import type { TrayItem } from './tray';
import { tuckedBox } from './tuck';
import { type Stamp, toneColour } from './verdict';

export interface PanelChrome {
  label: string;
  name: string;
  prediction: string;
}

export interface PanelData {
  chrome: PanelChrome;
  setup: Setup;
  links: Link[];
  items: TrayItem[];
}

export type GridCtx = PanelCtx & Pick<CanvasRenderingContext2D, 'getImageData' | 'putImageData'>;

// What the view needs from a canvas, so a test can hand it a fake.
export interface GridCanvas {
  width: number;
  height: number;
  style: { width: string; height: string; imageRendering: string };
  getContext(contextId: '2d', options?: CanvasRenderingContext2DSettings): GridCtx | null;
}

const BAND = '#141a27';
const EDGE = '#3d5a80';
const LABEL = '#dfe6ee';
const NAME = '#e2b53c';
const PREDICTION = '#b9c0c9';
// The machine name is drawn at twice the font size when it fits the panel that way.
const BIG_NAME_CHARS = 33;
const RGB = rgbPalette(PALETTE);
const LUT = paletteLut(RGB);

export class ArenaView {
  grid: Size;
  private readonly panel: Size;
  private cols = COLS;
  private rects: Rect[];
  private views: PanelView[];
  private chromeDrawn = false;

  constructor(
    private readonly canvas: GridCanvas,
    private readonly panels: PanelData[],
    world_w: number,
    world_h: number,
  ) {
    this.panel = panelSize(world_w, world_h);
    this.rects = panelRects(panels.length, this.cols, this.panel);
    this.grid = gridSize(panels.length, this.cols, this.panel);
    this.views = this.makeViews();
  }

  private makeViews(): PanelView[] {
    return this.panels.map((p, i) => {
      const rect = this.rects[i] ?? { x: 0, y: 0, w: this.panel.w, h: this.panel.h };
      return new PanelView(p.setup, p.links, p.items, worldRect(rect));
    });
  }

  // Lays the panels out in `cols` columns, when that differs from the current layout.
  private relayout(cols: number): void {
    if (cols === this.cols) {
      return;
    }
    this.cols = cols;
    this.rects = panelRects(this.panels.length, cols, this.panel);
    this.grid = gridSize(this.panels.length, cols, this.panel);
    this.views = this.makeViews();
  }

  // Picks the columns that show the grid biggest in the box, then sizes the backing store to the
  // grid and the element to the upscale that fits.
  resize(css_w: number, css_h: number): number {
    this.relayout(bestColumns(this.panels.length, css_w, css_h, this.panel));
    const scale = fitScale(css_w, css_h, this.grid);
    this.canvas.width = this.grid.w;
    this.canvas.height = this.grid.h;
    this.canvas.style.width = `${this.grid.w * scale}px`;
    this.canvas.style.height = `${this.grid.h * scale}px`;
    this.canvas.style.imageRendering = 'pixelated';
    this.chromeDrawn = false;
    return scale;
  }

  // Repaints every panel given a frame; `null` leaves that panel as it is.
  draw(frames: (PanelFrame | null)[]): void {
    const ctx = this.canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) {
      return;
    }
    if (!this.chromeDrawn) {
      this.panels.forEach((p, i) => {
        const rect = this.rects[i];
        if (rect) {
          this.chrome(ctx, rect, p.chrome);
        }
      });
      this.chromeDrawn = true;
    }
    frames.forEach((frame, i) => {
      const view = this.views[i];
      const rect = this.rects[i];
      const p = this.panels[i];
      if (!frame || !view || !rect || !p) {
        return;
      }
      view.draw(ctx, frame);
      const r = view.rect;
      const image = ctx.getImageData(r.x, r.y, r.w, r.h);
      snapToPalette(image.data, RGB, LUT);
      ctx.putImageData(image, r.x, r.y);
      if (frame.stamp !== null && frame.tucked) {
        this.tuckedStamp(ctx, headerRect(rect), legible(p.chrome.label), frame.stamp);
      }
    });
  }

  // The stamp in small on the header's first line: paper, a border in the verdict's colour, pixel text.
  private tuckedStamp(ctx: GridCtx, header: Rect, label: string, stamp: Stamp): void {
    const b = tuckedBox(header, label, stamp.text);
    const colour = toneColour(stamp.tone);
    ctx.fillStyle = colour;
    ctx.fillRect(b.x, b.y, b.w, b.h);
    ctx.fillStyle = PAPER;
    ctx.fillRect(b.x + b.px, b.y + b.px, b.w - 2 * b.px, b.h - 2 * b.px);
    drawText(ctx, stamp.text, b.x + 2 * b.px, b.y + b.px, b.px, colour);
  }

  private chrome(ctx: GridCtx, rect: Rect, chrome: PanelChrome): void {
    const header = headerRect(rect);
    const footer = footerRect(rect);
    ctx.fillStyle = BAND;
    ctx.fillRect(header.x, header.y, header.w, header.h);
    ctx.fillRect(footer.x, footer.y, footer.w, footer.h);
    const label = legible(chrome.label);
    drawText(ctx, label, header.x + 4, header.y + 4, 2, LABEL);
    const name = legible(chrome.name);
    if (name.length <= BIG_NAME_CHARS) {
      drawText(ctx, name, header.x + 4, header.y + 4 + GLYPH_H * 2 + 4, 2, NAME);
    } else {
      drawText(ctx, name, header.x + 4, header.y + 4 + GLYPH_H * 2 + 6, 1, NAME);
    }
    wrapText(legible(chrome.prediction), FOOTER_CHARS, FOOTER_LINES).forEach((line, i) => {
      drawText(ctx, line, footer.x + 2, footer.y + 2 + i * (GLYPH_H + 1), 1, PREDICTION);
    });
    ctx.strokeStyle = EDGE;
    ctx.lineWidth = 1;
    ctx.strokeRect(rect.x + 0.5, rect.y + 0.5, rect.w - 1, rect.h - 1);
  }
}
