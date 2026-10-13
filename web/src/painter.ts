// Draws one body from its Visual and Pose. Everything is drawn in code in a local frame measured in
// metres with y up, so sizes stay true at any zoom. Shapes match the colliders; looks add the finish.

import type { Camera } from './camera';
import type { Look, Pose, Visual } from './decode';

export type Ctx2D = Pick<
  CanvasRenderingContext2D,
  | 'save'
  | 'restore'
  | 'translate'
  | 'rotate'
  | 'scale'
  | 'beginPath'
  | 'closePath'
  | 'moveTo'
  | 'lineTo'
  | 'arc'
  | 'ellipse'
  | 'rect'
  | 'fill'
  | 'stroke'
  | 'fillRect'
  | 'strokeRect'
  | 'setLineDash'
  | 'createLinearGradient'
  | 'createRadialGradient'
  | 'fillStyle'
  | 'strokeStyle'
  | 'lineWidth'
  | 'lineCap'
  | 'lineJoin'
  | 'globalAlpha'
>;

const LINE_M = 0.008;

interface Finish {
  fill: string;
  edge: string;
  detail: string;
}

const FINISH: Record<Look, Finish> = {
  wood: { fill: '#c8955a', edge: '#6f4a26', detail: '#a9773f' },
  steel: { fill: '#b9c0c9', edge: '#444a53', detail: '#f4f6f8' },
  rubber: { fill: '#2f2f33', edge: '#111113', detail: '#55555b' },
  domino: { fill: '#f3ead7', edge: '#2b2b2b', detail: '#2b2b2b' },
  balloon: { fill: '#d92626', edge: '#8f1414', detail: '#ffb3b3' },
  pad: { fill: '#6b7480', edge: '#2d323a', detail: '#3c424b' },
  pin: { fill: '#d8dde3', edge: '#3a3f46', detail: '#8a9099' },
  brass: { fill: '#e2b53c', edge: '#6b4f0a', detail: '#fff0b8' },
  wicker: { fill: '#c9a46a', edge: '#6e4e22', detail: '#8f6a35' },
};

export class PartPainter {
  draw(ctx: Ctx2D, camera: Camera, visual: Visual, pose: Pose): void {
    if (!pose.alive) {
      return;
    }
    const at = camera.toPx(pose.x_m, pose.y_m);
    ctx.save();
    ctx.translate(at.x, at.y);
    ctx.rotate(-pose.angle_rad);
    ctx.scale(camera.scale_px_per_m, -camera.scale_px_per_m);
    ctx.lineWidth = LINE_M;
    ctx.lineJoin = 'round';
    switch (visual.shape) {
      case 'circle':
        this.circle(ctx, visual.r_m, visual.look);
        break;
      case 'box':
        this.box(ctx, visual.w_m, visual.h_m, visual.look);
        break;
      case 'trough':
        this.trough(ctx, visual.inner_w_m, visual.inner_h_m, visual.wall_m, visual.look);
        break;
      case 'pivot':
        this.pivot(ctx);
        break;
    }
    ctx.restore();
  }

  // A rod or rope between two body centres.
  drawLink(ctx: Ctx2D, camera: Camera, from: Pose, to: Pose): void {
    if (!from.alive || !to.alive) {
      return;
    }
    const a = camera.toPx(from.x_m, from.y_m);
    const b = camera.toPx(to.x_m, to.y_m);
    ctx.save();
    ctx.strokeStyle = '#4a3a28';
    ctx.lineWidth = Math.max(1, camera.px(0.006));
    ctx.lineCap = 'round';
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.stroke();
    ctx.restore();
  }

  private circle(ctx: Ctx2D, r: number, look: Look): void {
    const f = FINISH[look];
    ctx.fillStyle = look === 'steel' || look === 'brass' || look === 'balloon' ? shine(ctx, r, f) : f.fill;
    ctx.strokeStyle = f.edge;
    disc(ctx, 0, 0, r);
    ctx.fill();
    ctx.stroke();
    switch (look) {
      case 'wood':
        ctx.strokeStyle = f.detail;
        ctx.lineWidth = LINE_M / 2;
        disc(ctx, 0, 0, r * 0.62);
        ctx.stroke();
        disc(ctx, 0, 0, r * 0.3);
        ctx.stroke();
        break;
      case 'rubber':
        ctx.strokeStyle = f.detail;
        ctx.lineWidth = LINE_M / 2;
        disc(ctx, 0, 0, r * 0.8);
        ctx.stroke();
        break;
      case 'balloon':
        this.balloonKnot(ctx, r, f);
        break;
      case 'brass':
        ctx.strokeStyle = f.edge;
        ctx.lineWidth = LINE_M / 2;
        disc(ctx, 0, 0, r * 0.78);
        ctx.stroke();
        ctx.fillStyle = '#5a3d05';
        disc(ctx, 0, -r * 0.62, r * 0.16);
        ctx.fill();
        break;
      default:
        break;
    }
    if (look === 'steel' || look === 'brass' || look === 'balloon') {
      ctx.fillStyle = f.detail;
      ctx.globalAlpha = 0.55;
      ctx.beginPath();
      ctx.ellipse(-r * 0.35, r * 0.35, r * 0.22, r * 0.13, Math.PI / 4, 0, Math.PI * 2);
      ctx.fill();
      ctx.globalAlpha = 1;
    }
  }

  private balloonKnot(ctx: Ctx2D, r: number, f: Finish): void {
    ctx.fillStyle = f.edge;
    ctx.beginPath();
    ctx.moveTo(0, -r * 0.92);
    ctx.lineTo(-r * 0.16, -r * 1.14);
    ctx.lineTo(r * 0.16, -r * 1.14);
    ctx.closePath();
    ctx.fill();
  }

  private box(ctx: Ctx2D, w: number, h: number, look: Look): void {
    const f = FINISH[look];
    if (look === 'pin') {
      this.needle(ctx, w, h, f);
      return;
    }
    if (look === 'pad') {
      this.spring(ctx, w, h, f);
    }
    ctx.fillStyle = look === 'steel' ? shine(ctx, Math.max(w, h) / 2, f) : f.fill;
    ctx.strokeStyle = f.edge;
    ctx.fillRect(-w / 2, -h / 2, w, h);
    ctx.strokeRect(-w / 2, -h / 2, w, h);
    ctx.strokeStyle = f.detail;
    ctx.lineWidth = LINE_M / 2;
    switch (look) {
      case 'wood':
        this.grain(ctx, w, h);
        break;
      case 'domino':
        line(ctx, -w / 2, 0, w / 2, 0);
        ctx.fillStyle = f.detail;
        disc(ctx, 0, h / 4, Math.min(w, h) * 0.18);
        ctx.fill();
        disc(ctx, 0, -h / 4, Math.min(w, h) * 0.18);
        ctx.fill();
        break;
      default:
        break;
    }
  }

  // Two lines along the long axis, inset a quarter of the thickness.
  private grain(ctx: Ctx2D, w: number, h: number): void {
    if (w >= h) {
      line(ctx, -w / 2 + h / 4, h / 4, w / 2 - h / 4, h / 4);
      line(ctx, -w / 2 + h / 4, -h / 4, w / 2 - h / 4, -h / 4);
    } else {
      line(ctx, w / 4, -h / 2 + w / 4, w / 4, h / 2 - w / 4);
      line(ctx, -w / 4, -h / 2 + w / 4, -w / 4, h / 2 - w / 4);
    }
  }

  // The pin's long axis is local x and its tip is at +x.
  private needle(ctx: Ctx2D, w: number, h: number, f: Finish): void {
    const tip = h * 2;
    ctx.fillStyle = f.fill;
    ctx.strokeStyle = f.edge;
    ctx.beginPath();
    ctx.moveTo(-w / 2, -h / 2);
    ctx.lineTo(w / 2 - tip, -h / 2);
    ctx.lineTo(w / 2, 0);
    ctx.lineTo(w / 2 - tip, h / 2);
    ctx.lineTo(-w / 2, h / 2);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = f.detail;
    disc(ctx, -w / 2, 0, h);
    ctx.fill();
    ctx.stroke();
  }

  // A zigzag coil under the plate, on the side away from the launch face.
  private spring(ctx: Ctx2D, w: number, h: number, f: Finish): void {
    const depth = h * 1.5;
    const top = -h / 2;
    const zigs = 4;
    ctx.strokeStyle = f.detail;
    ctx.lineWidth = LINE_M;
    for (const cx of [-w / 3, w / 3]) {
      ctx.beginPath();
      ctx.moveTo(cx, top);
      for (let i = 1; i <= zigs; i += 1) {
        const side = i % 2 === 0 ? -1 : 1;
        ctx.lineTo(cx + side * h * 0.6, top - (depth * i) / zigs);
      }
      ctx.stroke();
    }
  }

  // Origin is the floor's underside centre; walls rise from the floor's ends.
  private trough(ctx: Ctx2D, inner_w: number, inner_h: number, wall: number, look: Look): void {
    const f = FINISH[look];
    const outer = inner_w / 2 + wall;
    ctx.fillStyle = f.fill;
    ctx.strokeStyle = f.edge;
    const pieces: [number, number, number, number][] = [
      [-outer, 0, outer * 2, wall],
      [-outer, wall, wall, inner_h],
      [inner_w / 2, wall, wall, inner_h],
    ];
    for (const [x, y, pw, ph] of pieces) {
      ctx.fillRect(x, y, pw, ph);
      ctx.strokeRect(x, y, pw, ph);
    }
    ctx.strokeStyle = f.detail;
    ctx.lineWidth = LINE_M / 2;
    if (look === 'wicker') {
      const pitch = Math.max(wall, 0.02);
      for (let y = wall + pitch; y < wall + inner_h; y += pitch) {
        line(ctx, -outer, y, -inner_w / 2, y);
        line(ctx, inner_w / 2, y, outer, y);
      }
      for (let x = -outer + pitch; x < outer; x += pitch) {
        line(ctx, x, 0, x, wall);
      }
    } else {
      line(ctx, -outer + wall, wall / 2, outer - wall, wall / 2);
    }
  }

  private pivot(ctx: Ctx2D): void {
    ctx.fillStyle = '#3a3a3a';
    ctx.strokeStyle = '#111';
    disc(ctx, 0, 0, 0.012);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = '#a0a0a0';
    disc(ctx, 0, 0, 0.004);
    ctx.fill();
  }
}

function disc(ctx: Ctx2D, x: number, y: number, r: number): void {
  ctx.beginPath();
  ctx.arc(x, y, r, 0, Math.PI * 2);
}

function line(ctx: Ctx2D, x0: number, y0: number, x1: number, y1: number): void {
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(x1, y1);
  ctx.stroke();
}

// A radial highlight up and to the left, for polished looks.
function shine(ctx: Ctx2D, r: number, f: Finish): CanvasGradient {
  const g = ctx.createRadialGradient(-r * 0.4, r * 0.4, r * 0.1, 0, 0, r * 1.2);
  g.addColorStop(0, f.detail);
  g.addColorStop(0.35, f.fill);
  g.addColorStop(1, f.edge);
  return g;
}
