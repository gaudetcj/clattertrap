// The editor's canvas: one arena panel showing a scene as built for drawing, with an outline round
// the picked part, and the mapping from a pointer on the page to a point in the world.

import { ArenaView, type PanelChrome } from './arena_view';
import { Camera } from './camera';
import type { Pose } from './decode';
import { roomFor } from './fit';
import { boundsOf, partAt } from './hit';
import { panelSize, worldRect, type Rect } from './layout';
import type { Replay } from './replay';
import { tray } from './tray';
import { TRAY_H } from './tray_view';

const OUTLINE = '#e2b53c';
const OUTLINE_PAD_PX = 3;

export class EditorView {
  private view: ArenaView | null = null;
  private replay: Replay | null = null;
  private poses: Pose[] = [];
  private world: Rect = { x: 0, y: 0, w: 1, h: 1 };
  private camera: Camera | null = null;
  private picked: string | null = null;

  constructor(private readonly canvas: HTMLCanvasElement) {}

  // Shows a built scene at its start, outlining the part named `picked` when there is one.
  show(replay: Replay, levelJson: string, sceneJson: string, chrome: PanelChrome, picked: string | null): void {
    this.replay?.free();
    this.replay = replay;
    this.poses = replay.frame();
    this.picked = picked;
    const setup = replay.setup;
    const panel = panelSize(setup.width_cm, setup.height_cm);
    this.world = worldRect({ x: 0, y: 0, w: panel.w, h: panel.h });
    this.camera = Camera.fit(setup.width_cm, setup.height_cm, this.world.w, this.world.h - TRAY_H);
    this.view = new ArenaView(
      this.canvas,
      [{ chrome, setup, links: replay.links, items: tray(levelJson, sceneJson) }],
      setup.width_cm,
      setup.height_cm,
    );
    this.fit();
  }

  // Sizes the canvas to the room the page gives it and paints the scene again.
  fit(): void {
    if (!this.view) {
      return;
    }
    const room = roomFor(this.canvas);
    this.view.resize(room.w, room.h);
    this.paint();
  }

  // The replay on show, so the page can hand it on or free it.
  shown(): Replay | null {
    return this.replay;
  }

  // The scene part under a pointer, by name, or undefined over nothing or over a fixture.
  partUnder(e: MouseEvent, sceneIds: Set<string>): string | undefined {
    const at = this.worldAt(e);
    if (!at || !this.replay) {
      return undefined;
    }
    const slot = partAt(this.replay.setup, this.poses, at.x_m, at.y_m);
    const part = slot === undefined ? undefined : this.replay.setup.slots[slot]?.part;
    const name = part === undefined ? undefined : this.replay.setup.names[part];
    return name !== undefined && sceneIds.has(name) ? name : undefined;
  }

  // The world point under a pointer in centimetres, or undefined before a scene is shown.
  worldAt(e: MouseEvent): { x_m: number; y_m: number } | undefined {
    if (!this.camera) {
      return undefined;
    }
    const box = this.canvas.getBoundingClientRect();
    const x_px = ((e.clientX - box.left) * this.canvas.width) / box.width - this.world.x;
    const y_px = ((e.clientY - box.top) * this.canvas.height) / box.height - this.world.y;
    return this.camera.toWorld(x_px, y_px);
  }

  private paint(): void {
    if (!this.view || !this.replay) {
      return;
    }
    this.view.draw([{ poses: this.poses, time_s: 0, stamp: null, tucked: false }]);
    if (this.picked !== null) {
      this.outline(this.picked);
    }
  }

  // A dashed box round every body of the picked part.
  private outline(name: string): void {
    const ctx = this.canvas.getContext('2d');
    const camera = this.camera;
    if (!ctx || !camera || !this.replay) {
      return;
    }
    const setup = this.replay.setup;
    const part = setup.names.indexOf(name);
    let box: { min_x: number; min_y: number; max_x: number; max_y: number } | null = null;
    setup.slots.forEach((slot, i) => {
      const pose = this.poses[i];
      if (slot.part !== part || !pose || !pose.alive) {
        return;
      }
      const b = boundsOf(slot.visual, pose);
      box = box
        ? { min_x: Math.min(box.min_x, b.min_x), min_y: Math.min(box.min_y, b.min_y), max_x: Math.max(box.max_x, b.max_x), max_y: Math.max(box.max_y, b.max_y) }
        : b;
    });
    if (!box) {
      return;
    }
    const b: { min_x: number; min_y: number; max_x: number; max_y: number } = box;
    const a = camera.toPx(b.min_x, b.max_y);
    const z = camera.toPx(b.max_x, b.min_y);
    ctx.save();
    ctx.translate(this.world.x, this.world.y);
    ctx.strokeStyle = OUTLINE;
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 2]);
    ctx.strokeRect(
      Math.round(a.x) - OUTLINE_PAD_PX + 0.5,
      Math.round(a.y) - OUTLINE_PAD_PX + 0.5,
      Math.round(z.x - a.x) + 2 * OUTLINE_PAD_PX,
      Math.round(z.y - a.y) + 2 * OUTLINE_PAD_PX,
    );
    ctx.restore();
  }
}
