// Draws one level in 16-bit pixels: the background, build zones, ground line, every body, the rods
// and ropes, the clock, and the verdict stamp once the run has ended.

import { Camera } from './camera';
import type { Pose, Setup } from './decode';
import { drawText, GLYPH_H, textWidth } from './font';
import type { Link } from './links';
import { PartPainter } from './painter';
import { CM_PER_PX, PALETTE, paletteLut, pixelGrid, rgbPalette, snapToPalette } from './pixel';

const BACKGROUND = '#1b2233';
const ZONE_EDGE = '#3d5a80';
const ZONE_FILL = '#243149';
const CLOCK = '#dfe6ee';
const RGB = rgbPalette(PALETTE);
const LUT = paletteLut(RGB);

export class LevelView {
  private readonly painter = new PartPainter();
  private camera: Camera;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly setup: Setup,
    private readonly links: Link[],
  ) {
    this.camera = Camera.fit(setup.width_cm, setup.height_cm, canvas.width, canvas.height);
  }

  // Sizes the backing store to the low-res grid, which the browser upscales by a whole number
  // without smoothing, and refits the camera.
  resize(css_w: number, css_h: number): void {
    const grid = pixelGrid(css_w, css_h, this.setup.width_cm, CM_PER_PX);
    this.canvas.width = grid.w;
    this.canvas.height = grid.h;
    this.canvas.style.width = `${grid.w * grid.scale}px`;
    this.canvas.style.height = `${grid.h * grid.scale}px`;
    this.canvas.style.imageRendering = 'pixelated';
    this.camera = Camera.fit(this.setup.width_cm, this.setup.height_cm, grid.w, grid.h);
  }

  draw(poses: Pose[], time_s: number, stamp: string | null, success: boolean): void {
    // Every frame is read back for the palette snap.
    const ctx = this.canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) {
      return;
    }
    const w = this.canvas.width;
    const h = this.canvas.height;
    ctx.fillStyle = BACKGROUND;
    ctx.fillRect(0, 0, w, h);
    this.zones(ctx);
    this.ground(ctx);
    for (const link of this.links) {
      const from = poses[link.from];
      const to = poses[link.to];
      if (from && to) {
        this.painter.drawLink(ctx, this.camera, from, to);
      }
    }
    this.setup.slots.forEach((slot, i) => {
      const pose = poses[i];
      if (pose) {
        this.painter.draw(ctx, this.camera, slot.visual, pose);
      }
    });
    this.clock(ctx, time_s);
    if (stamp !== null) {
      this.stamp(ctx, w, h, stamp, success);
    }
    const frame = ctx.getImageData(0, 0, w, h);
    snapToPalette(frame.data, RGB, LUT);
    ctx.putImageData(frame, 0, 0);
  }

  private zones(ctx: CanvasRenderingContext2D): void {
    ctx.save();
    ctx.strokeStyle = ZONE_EDGE;
    ctx.fillStyle = ZONE_FILL;
    ctx.lineWidth = 1;
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

  private clock(ctx: CanvasRenderingContext2D, time_s: number): void {
    drawText(ctx, `${time_s.toFixed(2)} s`, 4, 4, 1, CLOCK);
  }

  // A straight banner across the middle: paper, a thick border in the verdict's colour, pixel text.
  private stamp(ctx: CanvasRenderingContext2D, w: number, h: number, text: string, success: boolean): void {
    const colour = success ? '#1f7a3a' : '#b3261e';
    const px = Math.max(2, Math.round(Math.min(w, h) / 12 / GLYPH_H));
    const pad = 3 * px;
    const width = textWidth(text, px) + pad * 2;
    const height = GLYPH_H * px + pad * 2;
    const x = Math.round((w - width) / 2);
    const y = Math.round((h - height) / 2);
    ctx.fillStyle = colour;
    ctx.fillRect(x - px, y - px, width + 2 * px, height + 2 * px);
    ctx.fillStyle = '#f6f1e7';
    ctx.fillRect(x, y, width, height);
    drawText(ctx, text, x + pad, y + pad, px, colour);
  }
}
