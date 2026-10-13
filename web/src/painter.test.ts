import { describe, expect, it } from 'vitest';
import { Camera } from './camera';
import { LOOKS, type Look, type Pose, type Visual } from './decode';
import { PartPainter, type Ctx2D } from './painter';

// Records what a painter asks of a 2D context, since node has no canvas.
class RecordingCtx {
  calls: string[] = [];
  fillStyle: string | CanvasGradient | CanvasPattern = '#000';
  strokeStyle: string | CanvasGradient | CanvasPattern = '#000';
  lineWidth = 1;
  lineCap: CanvasLineCap = 'butt';
  lineJoin: CanvasLineJoin = 'miter';
  globalAlpha = 1;
  font = '';
  textAlign: CanvasTextAlign = 'start';
  textBaseline: CanvasTextBaseline = 'alphabetic';
  private gradient(): CanvasGradient {
    return { addColorStop: () => undefined } as unknown as CanvasGradient;
  }
  save() { this.calls.push('save'); }
  restore() { this.calls.push('restore'); }
  translate() { this.calls.push('translate'); }
  rotate() { this.calls.push('rotate'); }
  scale() { this.calls.push('scale'); }
  beginPath() { this.calls.push('beginPath'); }
  closePath() { this.calls.push('closePath'); }
  moveTo() { this.calls.push('moveTo'); }
  lineTo() { this.calls.push('lineTo'); }
  arc() { this.calls.push('arc'); }
  ellipse() { this.calls.push('ellipse'); }
  rect() { this.calls.push('rect'); }
  fill() { this.calls.push('fill'); }
  stroke() { this.calls.push('stroke'); }
  fillRect() { this.calls.push('fillRect'); }
  strokeRect() { this.calls.push('strokeRect'); }
  fillText() { this.calls.push('fillText'); }
  setLineDash() { this.calls.push('setLineDash'); }
  createLinearGradient() { this.calls.push('createLinearGradient'); return this.gradient(); }
  createRadialGradient() { this.calls.push('createRadialGradient'); return this.gradient(); }
  count(name: string): number { return this.calls.filter((c) => c === name).length; }
}

const camera = Camera.fit(400, 300, 800, 600);
const pose: Pose = { x_m: 1, y_m: 1, angle_rad: 0.3, alive: true };

function visuals(): Visual[] {
  const out: Visual[] = [{ shape: 'pivot' }];
  for (const look of LOOKS) {
    out.push({ shape: 'circle', r_m: 0.06, look });
    out.push({ shape: 'box', w_m: 0.4, h_m: 0.04, look });
  }
  const troughLooks: Look[] = ['wood', 'wicker'];
  for (const look of troughLooks) {
    out.push({ shape: 'trough', inner_w_m: 0.3, inner_h_m: 0.2, wall_m: 0.02, look });
  }
  return out;
}

describe('PartPainter', () => {
  it('paints every shape and look with balanced state and at least one fill', () => {
    const painter = new PartPainter();
    for (const visual of visuals()) {
      const ctx = new RecordingCtx();
      painter.draw(ctx as unknown as Ctx2D, camera, visual, pose);
      expect(ctx.count('save'), JSON.stringify(visual)).toBe(ctx.count('restore'));
      expect(ctx.count('fill') + ctx.count('fillRect'), JSON.stringify(visual)).toBeGreaterThan(0);
    }
  });

  it('draws a link as one stroked line', () => {
    const ctx = new RecordingCtx();
    new PartPainter().drawLink(ctx as unknown as Ctx2D, camera, pose, { x_m: 2, y_m: 2, angle_rad: 0, alive: true });
    expect(ctx.count('moveTo')).toBe(1);
    expect(ctx.count('lineTo')).toBe(1);
    expect(ctx.count('stroke')).toBe(1);
  });

  it('skips a body that no longer exists', () => {
    const ctx = new RecordingCtx();
    new PartPainter().draw(ctx as unknown as Ctx2D, camera, { shape: 'circle', r_m: 0.1, look: 'balloon' }, { ...pose, alive: false });
    expect(ctx.calls).toHaveLength(0);
  });
});
