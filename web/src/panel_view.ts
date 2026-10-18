// Draws one machine into its rect on a shared canvas: the background, build zones, ground line,
// every body, the rods and ropes, the inventory tray, the clock, and the stamp once the run has ended.

import { Camera } from './camera';
import type { Pose, Setup } from './decode';
import { drawText, GLYPH_H, textWidth } from './font';
import type { Rect } from './layout';
import type { Link } from './links';
import { PartPainter, type Ctx2D } from './painter';
import type { TrayItem } from './tray';
import { drawTray, TRAY_H } from './tray_view';
import { type Stamp, toneColour } from './verdict';

export type PanelCtx = Ctx2D & Pick<CanvasRenderingContext2D, 'clip'>;

export interface PanelFrame {
  poses: Pose[];
  time_s: number;
  stamp: Stamp | null;
}

export const BACKGROUND = '#1b2233';
const ZONE_EDGE = '#3d5a80';
const ZONE_FILL = '#243149';
const CLOCK = '#dfe6ee';
const GROUND = '#5b4327';
const PAPER = '#f6f1e7';

export class PanelView {
  private readonly painter = new PartPainter();
  private readonly camera: Camera;

  // `rect` is the world and its tray, in canvas pixels.
  constructor(
    private readonly setup: Setup,
    private readonly links: Link[],
    private readonly items: TrayItem[],
    readonly rect: Rect,
  ) {
    this.camera = Camera.fit(setup.width_cm, setup.height_cm, rect.w, rect.h - TRAY_H);
  }

  draw(ctx: PanelCtx, frame: PanelFrame): void {
    const { w, h } = this.rect;
    ctx.save();
    ctx.translate(this.rect.x, this.rect.y);
    ctx.beginPath();
    ctx.rect(0, 0, w, h);
    ctx.clip();
    ctx.fillStyle = BACKGROUND;
    ctx.fillRect(0, 0, w, h);
    this.zones(ctx);
    this.ground(ctx);
    for (const link of this.links) {
      const from = frame.poses[link.from];
      const to = frame.poses[link.to];
      if (from && to) {
        this.painter.drawLink(ctx, this.camera, from, to);
      }
    }
    this.setup.slots.forEach((slot, i) => {
      const pose = frame.poses[i];
      if (pose) {
        this.painter.draw(ctx, this.camera, slot.visual, pose);
      }
    });
    drawTray(ctx, this.painter, this.items, h - TRAY_H, w);
    drawText(ctx, `${frame.time_s.toFixed(2)} s`, 4, 4, 1, CLOCK);
    if (frame.stamp !== null) {
      this.stamp(ctx, w, h - TRAY_H, frame.stamp);
    }
    ctx.restore();
  }

  private zones(ctx: Ctx2D): void {
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

  private ground(ctx: Ctx2D): void {
    const a = this.camera.toPx(0, 0);
    const b = this.camera.toPx(this.setup.width_cm / 100, 0);
    ctx.save();
    ctx.strokeStyle = GROUND;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(a.x, a.y - 1);
    ctx.lineTo(b.x, b.y - 1);
    ctx.stroke();
    ctx.restore();
  }

  // A straight banner across the middle: paper, a thick border in the verdict's colour, pixel text.
  private stamp(ctx: Ctx2D, w: number, h: number, stamp: Stamp): void {
    const colour = toneColour(stamp.tone);
    const px = Math.max(2, Math.round(Math.min(w, h) / 12 / GLYPH_H));
    const pad = 3 * px;
    const width = textWidth(stamp.text, px) + pad * 2;
    const height = GLYPH_H * px + pad * 2;
    const x = Math.round((w - width) / 2);
    const y = Math.round((h - height) / 2);
    ctx.fillStyle = colour;
    ctx.fillRect(x - px, y - px, width + 2 * px, height + 2 * px);
    ctx.fillStyle = PAPER;
    ctx.fillRect(x, y, width, height);
    drawText(ctx, stamp.text, x + pad, y + pad, px, colour);
  }
}
