// Draws one level: the paper background, build zones, ground line, every body, the rods and ropes,
// the clock, and the verdict stamp once the run has ended.

import { Camera } from './camera';
import type { Pose, Setup } from './decode';
import type { Link } from './links';
import { PartPainter } from './painter';

export class LevelView {
  private readonly painter = new PartPainter();
  private camera: Camera;
  private dpr = 1;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly setup: Setup,
    private readonly links: Link[],
  ) {
    this.camera = Camera.fit(setup.width_cm, setup.height_cm, canvas.width, canvas.height);
  }

  // Sizes the backing store for the device so lines stay crisp, and refits the camera.
  resize(css_w: number, css_h: number): void {
    this.dpr = window.devicePixelRatio || 1;
    this.canvas.width = Math.round(css_w * this.dpr);
    this.canvas.height = Math.round(css_h * this.dpr);
    this.canvas.style.width = `${css_w}px`;
    this.canvas.style.height = `${css_h}px`;
    this.camera = Camera.fit(this.setup.width_cm, this.setup.height_cm, css_w, css_h);
  }

  draw(poses: Pose[], time_s: number, stamp: string | null, success: boolean): void {
    const ctx = this.canvas.getContext('2d');
    if (!ctx) {
      return;
    }
    ctx.save();
    ctx.scale(this.dpr, this.dpr);
    const w = this.canvas.width / this.dpr;
    const h = this.canvas.height / this.dpr;
    ctx.fillStyle = '#f6f1e7';
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
    ctx.restore();
  }

  private zones(ctx: CanvasRenderingContext2D): void {
    ctx.save();
    ctx.strokeStyle = '#8fa6bd';
    ctx.fillStyle = 'rgba(143, 166, 189, 0.08)';
    ctx.lineWidth = 1;
    ctx.setLineDash([6, 4]);
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
    ctx.save();
    ctx.fillStyle = '#4a4a4a';
    ctx.font = '14px ui-monospace, Consolas, monospace';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'top';
    ctx.fillText(`${time_s.toFixed(2)} s`, 10, 8);
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
    const width = ctx.measureText(text).width + 40;
    const height = Math.min(w, h) / 6;
    ctx.globalAlpha = 0.9;
    ctx.fillStyle = '#f6f1e7';
    ctx.fillRect(-width / 2, -height / 2, width, height);
    ctx.strokeStyle = colour;
    ctx.lineWidth = 4;
    ctx.strokeRect(-width / 2, -height / 2, width, height);
    ctx.fillStyle = colour;
    ctx.fillText(text, 0, 0);
    ctx.restore();
  }
}
