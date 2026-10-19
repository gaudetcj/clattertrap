import { describe, expect, it } from 'vitest';
import { ArenaView, type GridCanvas, type GridCtx, type PanelData } from './arena_view';
import type { Setup } from './decode';

// Records what the view asks of a canvas, since node has none.
class FakeCtx {
  calls: string[] = [];
  reads: [number, number, number, number][] = [];
  fillStyle: string | CanvasGradient | CanvasPattern = '#000';
  strokeStyle: string | CanvasGradient | CanvasPattern = '#000';
  lineWidth = 1;
  lineCap: CanvasLineCap = 'butt';
  lineJoin: CanvasLineJoin = 'miter';
  globalAlpha = 1;
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
  clip() { this.calls.push('clip'); }
  fill() { this.calls.push('fill'); }
  stroke() { this.calls.push('stroke'); }
  fillRect() { this.calls.push('fillRect'); }
  strokeRect() { this.calls.push('strokeRect'); }
  setLineDash() { this.calls.push('setLineDash'); }
  createLinearGradient() { this.calls.push('createLinearGradient'); return this.gradient(); }
  createRadialGradient() { this.calls.push('createRadialGradient'); return this.gradient(); }
  getImageData(x: number, y: number, w: number, h: number): ImageData {
    this.reads.push([x, y, w, h]);
    return { data: new Uint8ClampedArray(w * h * 4), width: w, height: h, colorSpace: 'srgb' } as ImageData;
  }
  putImageData() { this.calls.push('putImageData'); }
  count(name: string): number { return this.calls.filter((c) => c === name).length; }
}

class FakeCanvas implements GridCanvas {
  width = 0;
  height = 0;
  style = { width: '', height: '', imageRendering: '' };
  ctx = new FakeCtx();
  getContext(): GridCtx {
    return this.ctx as unknown as GridCtx;
  }
}

const setup: Setup = {
  width_cm: 400,
  height_cm: 300,
  time_limit_s: 20,
  zones: [{ x_cm: 0, y_cm: 0, width_cm: 400, height_cm: 300 }],
  goal: { kind: 'ring', target: 'bell', via: [] },
  names: ['bell', 'ball'],
  slots: [
    { part: 0, visual: { shape: 'circle', r_m: 0.1, look: 'brass' } },
    { part: 1, visual: { shape: 'circle', r_m: 0.05, look: 'steel' } },
  ],
};

function panel(label: string): PanelData {
  return {
    chrome: { label, name: 'Swatter Seesaw', prediction: 'The ball drops onto the seesaw and the bell rings.' },
    setup,
    links: [],
    items: [{ kind: 'seesaw', allowed: 1, used: 1 }],
  };
}

const frame = {
  poses: [
    { x_m: 3.6, y_m: 1, angle_rad: 0, alive: true },
    { x_m: 1, y_m: 2.8, angle_rad: 0, alive: true },
  ],
  time_s: 0.5,
  stamp: null,
};

describe('ArenaView', () => {
  it('sizes the canvas to the grid and the element to a whole-number multiple', () => {
    const canvas = new FakeCanvas();
    const view = new ArenaView(canvas, [panel('OPUS 5.5'), panel('HAIKU 5.5')], 400, 300);
    expect(view.grid).toEqual({ w: 800, h: 404 });
    expect(view.resize(1280, 900)).toBe(1);
    expect([canvas.width, canvas.height]).toEqual([800, 404]);
    expect(canvas.style).toEqual({ width: '800px', height: '404px', imageRendering: 'pixelated' });
    expect(view.resize(2560, 1300)).toBe(3);
    expect(canvas.style.width).toBe('2400px');
  });

  it('paints the chrome once and snaps only the panels that moved', () => {
    const canvas = new FakeCanvas();
    const view = new ArenaView(canvas, [panel('OPUS 5.5'), panel('HAIKU 5.5')], 400, 300);
    view.resize(1280, 900);
    view.draw([frame, null]);
    const chromeRects = canvas.ctx.count('fillRect');
    expect(chromeRects).toBeGreaterThan(0);
    expect(canvas.ctx.reads).toEqual([[0, 40, 400, 320]]);
    expect(canvas.ctx.count('putImageData')).toBe(1);
    canvas.ctx.calls = [];
    canvas.ctx.reads = [];
    view.draw([null, frame]);
    expect(canvas.ctx.reads).toEqual([[400, 40, 400, 320]]);
    expect(canvas.ctx.count('clip')).toBe(1);
    view.draw([null, null]);
    expect(canvas.ctx.reads).toHaveLength(1);
  });

  it('draws the stamp banner when a frame carries one', () => {
    const canvas = new FakeCanvas();
    const view = new ArenaView(canvas, [panel('OPUS 5.5')], 400, 300);
    view.resize(860, 900);
    view.draw([frame]);
    canvas.ctx.calls = [];
    view.draw([frame]);
    const plain = canvas.ctx.count('fillRect');
    canvas.ctx.calls = [];
    view.draw([{ ...frame, stamp: { text: 'RANG 0.7 s', tone: 'success' } }]);
    expect(canvas.ctx.count('fillRect')).toBeGreaterThan(plain);
  });
});

describe('ArenaView columns', () => {
  it('lays twelve lanes out four across on a desktop and six across on an ultrawide', () => {
    const canvas = new FakeCanvas();
    const lanes = Array.from({ length: 12 }, (_, i) => panel(`M${i}`));
    const view = new ArenaView(canvas, lanes, 400, 300);
    view.resize(1920, 1000);
    expect(view.grid).toEqual({ w: 1600, h: 1212 });
    expect([canvas.width, canvas.height]).toEqual([1600, 1212]);
    view.resize(3440, 900);
    expect(view.grid).toEqual({ w: 2400, h: 808 });
    view.draw(lanes.map(() => null));
    expect(canvas.ctx.count('putImageData')).toBe(0);
  });
});
