// Draws the inventory tray along the bottom of a panel: one cell per kind the level offered, with
// its icon and the count used over the count allowed.

import { Camera } from './camera';
import { drawText } from './font';
import type { Ctx2D, PartPainter } from './painter';
import { cellWidth, iconFor, type TrayItem } from './tray';

export const TRAY_H = 20;
const ICON_PX = 16;
const BAND = '#141a27';
const EDGE = '#3d5a80';
const USED = '#dfe6ee';
const UNUSED = '#8a9099';

// Icons are drawn true to the main view's scale: one pixel per centimetre.
const ICON_CAMERA = Camera.fit(ICON_PX, ICON_PX, ICON_PX, ICON_PX);

export function drawTray(ctx: Ctx2D, painter: PartPainter, items: TrayItem[], y: number, w: number): void {
  ctx.fillStyle = BAND;
  ctx.fillRect(0, y, w, TRAY_H);
  ctx.fillStyle = EDGE;
  ctx.fillRect(0, y, w, 1);
  const cell = cellWidth(items.length, w);
  items.forEach((item, i) => {
    const x = i * cell;
    const icon = iconFor(item.kind);
    ctx.save();
    // The icon camera's world origin is its bottom-left corner; this puts the icon's centre mid-cell.
    ctx.translate(x + ICON_PX / 2, y + TRAY_H / 2 - ICON_PX);
    for (const [from, to] of icon.links) {
      painter.drawLink(ctx, ICON_CAMERA, from, to);
    }
    for (const shape of icon.shapes) {
      painter.draw(ctx, ICON_CAMERA, shape.visual, shape.pose);
    }
    ctx.restore();
    drawText(ctx, `${item.used}/${item.allowed}`, x + ICON_PX + 5, y + (TRAY_H - 7) / 2 + 1, 1, item.used > 0 ? USED : UNUSED);
  });
}
