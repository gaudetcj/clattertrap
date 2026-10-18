// Draws one level: the background, build zones, ground line, every body, the rods and ropes, the
// clock, and the verdict stamp once the run has ended. Two looks: paper, and 16-bit pixels.

import { Camera } from './camera';
import type { Pose, Setup } from './decode';
import type { Link } from './links';
import { PartPainter } from './painter';
import { PALETTE, pixelGrid, rgbPalette, snapToPalette } from './pixel';

interface Theme {
  background: string;
  zoneEdge: string;
  zoneFill: string;
  clock: string;
  link: string;
}

const PAPER: Theme = {
  background: '#f6f1e7',
  zoneEdge: '#8fa6bd',
  zoneFill: 'rgba(143, 166, 189, 0.08)',
  clock: '#4a4a4a',
  link: '#4a3a28',
};
const ARCADE: Theme = { background: '#1b2233', zoneEdge: '#3d5a80', zoneFill: '#243149', clock: '#dfe6ee', link: '#c9a46a' };
const RGB = rgbPalette(PALETTE);

export class LevelView {
  private readonly painter = new PartPainter();
  private readonly snapped = new Map<number, number>();
  private camera: Camera;
  private dpr = 1;
  private scale = 1;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly setup: Setup,
    private readonly links: Link[],
    // Centimetres per pixel for the 16-bit look, or null for paper.
    private cm_per_px: number | null,
  ) {
    this.camera = Camera.fit(setup.width_cm, setup.height_cm, canvas.width, canvas.height);
  }

  setLook(cm_per_px: number | null): void {
    this.cm_per_px = cm_per_px;
  }

  // Sizes the backing store: the device's pixels for paper, or the low-res grid for 16-bit, which
  // the browser then upscales by a whole number without smoothing.
  resize(css_w: number, css_h: number): void {
    if (this.cm_per_px === null) {
      this.dpr = window.devicePixelRatio || 1;
      this.scale = 1;
      this.canvas.width = Math.round(css_w * this.dpr);
      this.canvas.height = Math.round(css_h * this.dpr);
      this.canvas.style.width = `${css_w}px`;
      this.canvas.style.height = `${css_h}px`;
      this.canvas.style.imageRendering = '';
      this.camera = Camera.fit(this.setup.width_cm, this.setup.height_cm, css_w, css_h);
      return;
    }
    const grid = pixelGrid(css_w, css_h, this.setup.width_cm, this.cm_per_px);
    this.dpr = 1;
    this.scale = grid.scale;
    this.canvas.width = grid.w;
    this.canvas.height = grid.h;
    this.canvas.style.width = `${grid.w * grid.scale}px`;
    this.canvas.style.height = `${grid.h * grid.scale}px`;
    this.canvas.style.imageRendering = 'pixelated';
    this.camera = Camera.fit(this.setup.width_cm, this.setup.height_cm, grid.w, grid.h);
  }

  draw(poses: Pose[], time_s: number, stamp: string | null, success: boolean): void {
    // The 16-bit look reads every frame back for the palette snap.
    const ctx = this.canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) {
      return;
    }
    const pixel = this.cm_per_px !== null;
    const theme = pixel ? ARCADE : PAPER;
    ctx.save();
    ctx.scale(this.dpr, this.dpr);
    const w = this.canvas.width / this.dpr;
    const h = this.canvas.height / this.dpr;
    ctx.fillStyle = theme.background;
    ctx.fillRect(0, 0, w, h);
    this.zones(ctx, theme, pixel);
    this.ground(ctx);
    for (const link of this.links) {
      const from = poses[link.from];
      const to = poses[link.to];
      if (from && to) {
        this.painter.drawLink(ctx, this.camera, from, to, theme.link);
      }
    }
    this.setup.slots.forEach((slot, i) => {
      const pose = poses[i];
      if (pose) {
        this.painter.draw(ctx, this.camera, slot.visual, pose);
      }
    });
    this.clock(ctx, time_s, theme);
    if (stamp !== null) {
      this.stamp(ctx, w, h, stamp, success);
    }
    ctx.restore();
    if (pixel) {
      const frame = ctx.getImageData(0, 0, this.canvas.width, this.canvas.height);
      snapToPalette(frame.data, RGB, this.snapped);
      ctx.putImageData(frame, 0, 0);
    }
  }

  private zones(ctx: CanvasRenderingContext2D, theme: Theme, pixel: boolean): void {
    ctx.save();
    ctx.strokeStyle = theme.zoneEdge;
    ctx.fillStyle = theme.zoneFill;
    ctx.lineWidth = 1;
    if (!pixel) {
      ctx.setLineDash([6, 4]);
    }
    for (const z of this.setup.zones) {
      const a = this.camera.toPx(z.x_cm / 100, (z.y_cm + z.height_cm) / 100);
      const wpx = this.camera.px(z.width_cm / 100);
      const hpx = this.camera.px(z.height_cm / 100);
      ctx.fillRect(a.x, a.y, wpx, hpx);
      ctx.strokeRect(a.x + 0.5, a.y + 0.5, wpx - 1, hpx - 1);
    }
    ctx.restore();
  }

  private ground(ctx: CanvasRenderingContext2D): void {
    const a = this.camera.toPx(0, 0);
    const b = this.camera.toPx(this.setup.width_cm / 100, 0);
    ctx.save();
    ctx.strokeStyle = '#5b4327';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(a.x, a.y - 1);
    ctx.lineTo(b.x, b.y - 1);
    ctx.stroke();
    ctx.restore();
  }

  private clock(ctx: CanvasRenderingContext2D, time_s: number, theme: Theme): void {
    ctx.save();
    ctx.fillStyle = theme.clock;
    const size = Math.max(8, Math.round(14 / this.scale));
    ctx.font = `${size}px ui-monospace, Consolas, monospace`;
    ctx.textAlign = 'left';
    ctx.textBaseline = 'top';
    ctx.fillText(`${time_s.toFixed(2)} s`, Math.round(10 / this.scale), Math.round(8 / this.scale));
    ctx.restore();
  }

  private stamp(ctx: CanvasRenderingContext2D, w: number, h: number, text: string, success: boolean): void {
    const colour = success ? '#1f7a3a' : '#b3261e';
    ctx.save();
    ctx.translate(w / 2, h / 2);
    ctx.rotate(-0.1);
    ctx.font = `bold ${Math.round(Math.min(w, h) / 9)}px Georgia, "Times New Roman", serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const width = ctx.measureText(text).width + 40 / this.scale;
    const height = Math.min(w, h) / 6;
    // Flat in the 16-bit look: a translucent paper over the dark background would snap to grey.
    ctx.globalAlpha = this.cm_per_px === null ? 0.9 : 1;
    ctx.fillStyle = '#f6f1e7';
    ctx.fillRect(-width / 2, -height / 2, width, height);
    ctx.strokeStyle = colour;
    ctx.lineWidth = Math.max(2, 4 / this.scale);
    ctx.strokeRect(-width / 2, -height / 2, width, height);
    ctx.fillStyle = colour;
    ctx.fillText(text, 0, 0);
    ctx.restore();
  }
}
