// Where a panel's stamp sits: it lands over the machine when the lane ends, holds, and then tucks
// into the header so the machine at rest can be seen.

import { GLYPH_H, textWidth } from './font';
import type { Rect } from './layout';

// How long a stamp lands over the machine before it tucks into the header, in seconds.
export const HOLD_S = 1.2;

export type Place = 'landed' | 'tucked';

export function stampPlace(age_s: number): Place {
  return age_s < HOLD_S ? 'landed' : 'tucked';
}

// A tucked stamp's box in canvas pixels, with its text scale.
export interface TuckedBox {
  x: number;
  y: number;
  w: number;
  h: number;
  px: number;
}

// The header's side margin, the gap kept after the label, and the label's text scale.
const MARGIN = 4;
const GAP = 8;
const LABEL_PX = 2;

function box(header: Rect, text: string, px: number): TuckedBox {
  const w = textWidth(text, px) + 4 * px;
  return { x: header.x + header.w - MARGIN - w, y: header.y + 2, w, h: GLYPH_H * px + 2 * px, px };
}

// The box on the first header line, right-aligned: at the label's scale when the label leaves room
// for it, else at the font size.
export function tuckedBox(header: Rect, label: string, text: string): TuckedBox {
  const big = box(header, text, LABEL_PX);
  const fits = MARGIN + textWidth(label, LABEL_PX) + GAP + big.w + MARGIN <= header.w;
  return fits ? big : box(header, text, 1);
}
