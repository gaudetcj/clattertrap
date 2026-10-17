// Reading a file dropped on the level page: a bare scene, or a pilot submission record wrapping one.

export interface Opened {
  levelId: string;
  sceneJson: string;
  name?: string;
  prediction?: string;
}

const NOT_A_SCENE = 'not a scene or a submission: expected {"level", "parts"} or {"scene": {"level", "parts"}}';

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

function isScene(v: unknown): v is { level: string; parts: unknown[] } {
  return isRecord(v) && typeof v['level'] === 'string' && Array.isArray(v['parts']);
}

// Throws when the text is not JSON or holds neither shape.
export function parseOpened(text: string): Opened {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (e) {
    throw new Error(`not JSON: ${e instanceof Error ? e.message : String(e)}`);
  }
  if (isScene(parsed)) {
    return { levelId: parsed.level, sceneJson: text };
  }
  if (isRecord(parsed) && isScene(parsed['scene'])) {
    const scene = parsed['scene'];
    const opened: Opened = { levelId: scene.level, sceneJson: JSON.stringify(scene) };
    if (typeof parsed['name'] === 'string') {
      opened.name = parsed['name'];
    }
    if (typeof parsed['prediction'] === 'string') {
      opened.prediction = parsed['prediction'];
    }
    return opened;
  }
  throw new Error(NOT_A_SCENE);
}
