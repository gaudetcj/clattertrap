// The scene a visitor builds: parts as the format writes them, edited by pure functions that return
// a new scene. Sizes come from the rules through the wasm, never from here.

// The part size ranges of the rules, in metres, as the wasm hands them out.
export interface Range {
  min_m: number;
  max_m: number;
}

export interface Ranges {
  beam_length: Range;
  seesaw_length: Range;
  crate_side: Range;
  pendulum_length: Range;
  bucket_size: Range;
  funnel_mouth: Range;
  funnel_height: Range;
  funnel_throat_min_m: number;
  rope_length: Range;
}

export interface EditPart {
  id: string;
  kind: string;
  [field: string]: number | string;
}

export interface EditScene {
  level: string;
  parts: EditPart[];
}

// A numeric field of a kind and the range the editor offers for it.
export interface Field {
  name: string;
  min: number;
  max: number;
  step: number;
}

export interface WorldSize {
  width_cm: number;
  height_cm: number;
}

// The letter each kind's ids start with.
const PREFIX: Record<string, string> = {
  beam: 'b',
  funnel: 'f',
  ball: 'k',
  domino: 'd',
  crate: 'c',
  seesaw: 's',
  pendulum: 'p',
  spring_pad: 'sp',
  bucket: 'u',
  balloon: 'o',
  pin: 'n',
  rope: 'r',
};

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

function cm(m: number): number {
  return Math.round(m * 1000) / 10;
}

// The first free id of a kind: its letter and a number from 1.
export function nextId(scene: EditScene, kind: string): string {
  const prefix = PREFIX[kind] ?? kind;
  const used = new Set(scene.parts.map((p) => p.id));
  let n = 1;
  while (used.has(`${prefix}${n}`)) {
    n += 1;
  }
  return `${prefix}${n}`;
}

// A part of a kind at a point, with the other fields at sensible defaults inside the rules.
export function defaultPart(kind: string, id: string, x_cm: number, y_cm: number, ranges: Ranges): EditPart {
  const at = { id, kind, x_cm, y_cm };
  switch (kind) {
    case 'beam':
      return { ...at, length_cm: 100, angle_deg: 0 };
    case 'funnel':
      return { ...at, mouth_cm: 80, throat_cm: 20, height_cm: 40 };
    case 'ball':
      return { ...at, material: 'steel' };
    case 'domino':
    case 'spring_pad':
      return { ...at, angle_deg: 0 };
    case 'crate':
      return { ...at, side_cm: 20 };
    case 'seesaw':
      return { ...at, length_cm: 100, angle_deg: 0 };
    case 'pendulum':
      return { ...at, length_cm: 60, angle_deg: 0 };
    case 'bucket':
      return { ...at, width_cm: 30, height_cm: 30 };
    case 'pin':
      return { ...at, angle_deg: -90 };
    case 'rope':
      return { id, kind, a: '', b: '', max_length_cm: cm(ranges.rope_length.max_m) / 2 };
    default:
      return at;
  }
}

function range(name: string, r: Range): Field {
  return { name, min: cm(r.min_m), max: cm(r.max_m), step: 1 };
}

const ANGLE: Field = { name: 'angle_deg', min: -180, max: 180, step: 1 };

// The numeric fields of a kind in the order the editor shows them: the centre, then the sizes, then
// the angle. A ball's material and a rope's ends are not numbers and are offered separately.
export function fieldsOf(kind: string, ranges: Ranges, world: WorldSize): Field[] {
  const centre: Field[] = [
    { name: 'x_cm', min: 0, max: world.width_cm, step: 1 },
    { name: 'y_cm', min: 0, max: world.height_cm, step: 1 },
  ];
  switch (kind) {
    case 'beam':
      return [...centre, range('length_cm', ranges.beam_length), ANGLE];
    case 'funnel':
      return [
        ...centre,
        range('mouth_cm', ranges.funnel_mouth),
        range('throat_cm', { min_m: ranges.funnel_throat_min_m, max_m: ranges.funnel_mouth.max_m }),
        range('height_cm', ranges.funnel_height),
      ];
    case 'domino':
    case 'spring_pad':
    case 'pin':
      return [...centre, ANGLE];
    case 'crate':
      return [...centre, range('side_cm', ranges.crate_side)];
    case 'seesaw':
      return [...centre, range('length_cm', ranges.seesaw_length), ANGLE];
    case 'pendulum':
      return [...centre, range('length_cm', ranges.pendulum_length), ANGLE];
    case 'bucket':
      return [...centre, range('width_cm', ranges.bucket_size), range('height_cm', ranges.bucket_size)];
    case 'rope':
      return [range('max_length_cm', ranges.rope_length)];
    default:
      return centre;
  }
}

export function addPart(scene: EditScene, part: EditPart): EditScene {
  return { ...scene, parts: [...scene.parts, part] };
}

function edit(scene: EditScene, id: string, change: (p: EditPart) => EditPart): EditScene {
  return { ...scene, parts: scene.parts.map((p) => (p.id === id ? change(p) : p)) };
}

// Moves a part's centre, to whole centimetres.
export function moveTo(scene: EditScene, id: string, x_cm: number, y_cm: number): EditScene {
  return edit(scene, id, (p) => ({ ...p, x_cm: Math.round(x_cm), y_cm: Math.round(y_cm) }));
}

export function setField(scene: EditScene, id: string, name: string, value: number | string): EditScene {
  return edit(scene, id, (p) => ({ ...p, [name]: value }));
}

// Removes a part and every rope tied to it.
export function removePart(scene: EditScene, id: string): EditScene {
  return { ...scene, parts: scene.parts.filter((p) => p.id !== id && !(p.kind === 'rope' && (p['a'] === id || p['b'] === id))) };
}

export function sceneJson(scene: EditScene): string {
  return JSON.stringify(scene);
}

// Throws when the text is not a scene: an object with a level id and parts that each have an id
// and a kind.
export function parseScene(text: string): EditScene {
  const v: unknown = JSON.parse(text);
  if (!isRecord(v) || typeof v['level'] !== 'string' || !Array.isArray(v['parts'])) {
    throw new Error('not a scene: expected {"level", "parts"}');
  }
  const parts = v['parts'].map((p: unknown, i): EditPart => {
    if (!isRecord(p) || typeof p['id'] !== 'string' || typeof p['kind'] !== 'string') {
      throw new Error(`parts[${i}]: expected an id and a kind`);
    }
    const part: EditPart = { id: p['id'], kind: p['kind'] };
    for (const [name, value] of Object.entries(p)) {
      if (typeof value === 'number' || typeof value === 'string') {
        part[name] = value;
      }
    }
    return part;
  });
  return { level: v['level'], parts };
}
