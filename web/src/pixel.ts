// The 16-bit look: a low-resolution frame, one pixel per centimetre of world, upscaled by a whole
// number with no smoothing, and every colour snapped to a fixed palette so edges stay hard.

export interface Grid {
  scale: number;
  w: number;
  h: number;
}

// One world centimetre per low-res pixel.
export const CM_PER_PX = 1;

// Every colour the painter and the view use, so a snapped frame keeps the materials readable.
export const PALETTE: string[] = [
  '#c8955a', '#6f4a26', '#a9773f',
  '#b9c0c9', '#444a53', '#f4f6f8',
  '#2f2f33', '#111113', '#55555b',
  '#f3ead7', '#2b2b2b',
  '#d92626', '#8f1414', '#ffb3b3',
  '#6b7480', '#2d323a', '#3c424b',
  '#d8dde3', '#3a3f46', '#8a9099',
  '#e2b53c', '#6b4f0a', '#fff0b8', '#5a3d05',
  '#c9a46a', '#6e4e22', '#8f6a35',
  '#3a3a3a', '#111111', '#a0a0a0', '#5b4327',
  '#1f7a3a', '#b3261e', '#f6f1e7',
  '#1b2233', '#3d5a80', '#243149', '#dfe6ee', '#141a27', '#141a27',
];

// The low-res size and whole-number upscale that puts about `cm_per_px` centimetres in one pixel.
export function pixelGrid(css_w: number, css_h: number, width_cm: number, cm_per_px: number): Grid {
  const scale = Math.max(1, Math.round(css_w / (width_cm / cm_per_px)));
  return { scale, w: Math.floor(css_w / scale), h: Math.floor(css_h / scale) };
}

// Hex colours as flat RGB triples.
export function rgbPalette(hex: string[]): Uint8Array {
  const out = new Uint8Array(hex.length * 3);
  hex.forEach((h, i) => {
    out[i * 3] = Number.parseInt(h.slice(1, 3), 16);
    out[i * 3 + 1] = Number.parseInt(h.slice(3, 5), 16);
    out[i * 3 + 2] = Number.parseInt(h.slice(5, 7), 16);
  });
  return out;
}

function nearest(r: number, g: number, b: number, palette: Uint8Array): number {
  let best = 0;
  let bestD = Number.POSITIVE_INFINITY;
  for (let i = 0; i < palette.length; i += 3) {
    const dr = r - (palette[i] ?? 0);
    const dg = g - (palette[i + 1] ?? 0);
    const db = b - (palette[i + 2] ?? 0);
    const d = dr * dr + dg * dg + db * db;
    if (d < bestD) {
      bestD = d;
      best = i / 3;
    }
  }
  return best;
}

// Five bits back to eight, so a bucket's representative sits where its exact colours do.
function expand(q: number): number {
  return (q << 3) | (q >> 2);
}

// The palette index nearest every five-bit-per-channel colour, built once per palette.
export function paletteLut(palette: Uint8Array): Uint8Array {
  const lut = new Uint8Array(32768);
  for (let key = 0; key < lut.length; key += 1) {
    lut[key] = nearest(expand(key >> 10), expand((key >> 5) & 31), expand(key & 31), palette);
  }
  return lut;
}

// Moves every RGBA pixel to its nearest palette colour through the table.
export function snapToPalette(data: Uint8ClampedArray, palette: Uint8Array, lut: Uint8Array): void {
  for (let i = 0; i < data.length; i += 4) {
    const key = (((data[i] ?? 0) >> 3) << 10) | (((data[i + 1] ?? 0) >> 3) << 5) | ((data[i + 2] ?? 0) >> 3);
    const at = (lut[key] ?? 0) * 3;
    data[i] = palette[at] ?? 0;
    data[i + 1] = palette[at + 1] ?? 0;
    data[i + 2] = palette[at + 2] ?? 0;
  }
}
