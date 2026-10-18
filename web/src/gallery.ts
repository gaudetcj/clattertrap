// The gallery bundled into every page: every entry under gallery/<level>/<model>/<arm>-NN, keyed
// by that path, with its submission and summary. Calls and transcripts load on demand elsewhere.

import { decodeOutcome, type Outcome } from './decode';

export interface EntryKey {
  level: string;
  model: string;
  run: string;
}

// What the pages show from a summary.
export interface Summary {
  status: string;
  cost_usd: number | null;
  duration_s: number;
}

export interface Entry {
  key: string;
  level: string;
  model: string;
  arm: string;
  run: string;
  name: string;
  prediction: string;
  sceneJson: string;
  outcome: Outcome;
  summary?: Summary;
}

const SUBMISSION = /gallery\/([^/]+)\/([^/]+)\/([^/]+)\/submission\.json$/;
const SUMMARY = /gallery\/([^/]+)\/([^/]+)\/([^/]+)\/summary\.json$/;
const CALL = /gallery\/([^/]+)\/([^/]+)\/([^/]+)\/calls\/(\d+)-([a-z]+)\.json$/;

// The six models of the first arena, in panel order; other ids follow them by name.
export const MODEL_ORDER: readonly string[] = [
  'claude-haiku-4-5',
  'claude-sonnet-4-5',
  'claude-opus-4-5',
  'claude-haiku-5-5',
  'claude-sonnet-5-5',
  'claude-opus-5-5',
];

function keyOf(m: RegExpExecArray | null): EntryKey | undefined {
  const [, level, model, run] = m ?? [];
  return level !== undefined && model !== undefined && run !== undefined ? { level, model, run } : undefined;
}

// The entry a submission path belongs to.
export function entryKey(path: string): EntryKey | undefined {
  return keyOf(SUBMISSION.exec(path));
}

export function keyString(k: EntryKey): string {
  return `${k.level}/${k.model}/${k.run}`;
}

// The entry, sequence, and tool of a call path.
export function callKey(path: string): { key: EntryKey; seq: number; tool: string } | undefined {
  const m = CALL.exec(path);
  const key = keyOf(m);
  const [, , , , seq, tool] = m ?? [];
  return key && seq !== undefined && tool !== undefined ? { key, seq: Number.parseInt(seq, 10), tool } : undefined;
}

// `claude-opus-5-5` reads `OPUS 5.5`: the family in capitals, the rest joined by dots.
export function modelLabel(id: string): string {
  const [family, ...rest] = id.replace(/^claude-/, '').split('-');
  const version = rest.join('.');
  return version.length === 0 ? (family ?? id).toUpperCase() : `${(family ?? id).toUpperCase()} ${version}`;
}

export function modelRank(id: string): number {
  const at = MODEL_ORDER.indexOf(id);
  return at === -1 ? MODEL_ORDER.length : at;
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

function readSummary(text: string | undefined): Summary | undefined {
  if (text === undefined) {
    return undefined;
  }
  try {
    const v: unknown = JSON.parse(text);
    if (!isRecord(v) || typeof v['status'] !== 'string' || typeof v['duration_s'] !== 'number') {
      return undefined;
    }
    return { status: v['status'], cost_usd: typeof v['cost_usd'] === 'number' ? v['cost_usd'] : null, duration_s: v['duration_s'] };
  } catch {
    return undefined;
  }
}

function readEntry(key: EntryKey, text: string, summary: string | undefined): Entry | undefined {
  try {
    const v: unknown = JSON.parse(text);
    if (!isRecord(v) || typeof v['name'] !== 'string' || typeof v['prediction'] !== 'string' || typeof v['arm'] !== 'string') {
      return undefined;
    }
    const entry: Entry = {
      key: keyString(key),
      level: key.level,
      model: key.model,
      arm: v['arm'],
      run: key.run,
      name: v['name'],
      prediction: v['prediction'],
      sceneJson: JSON.stringify(v['scene']),
      outcome: decodeOutcome(JSON.stringify(v['outcome'])),
    };
    const read = readSummary(summary);
    if (read !== undefined) {
      entry.summary = read;
    }
    return entry;
  } catch {
    return undefined;
  }
}

// Every readable submission in the globbed files, sorted by key; a record that cannot be read is
// left out with a warning.
export function indexGallery(submissions: Record<string, string>, summaries: Record<string, string>): Entry[] {
  const summaryByKey = new Map<string, string>();
  for (const [path, text] of Object.entries(summaries)) {
    const key = keyOf(SUMMARY.exec(path));
    if (key) {
      summaryByKey.set(keyString(key), text);
    }
  }
  const entries: Entry[] = [];
  for (const [path, text] of Object.entries(submissions)) {
    const key = entryKey(path);
    if (!key) {
      continue;
    }
    const entry = readEntry(key, text, summaryByKey.get(keyString(key)));
    if (entry) {
      entries.push(entry);
    } else {
      console.warn(`gallery: cannot read ${keyString(key)}`);
    }
  }
  return entries.sort((a, b) => a.key.localeCompare(b.key));
}

// One entry per model for a level and arm, the asked run number, in model order.
export function lineup(entries: Entry[], level: string, arm: string, run = '01'): Entry[] {
  return entries
    .filter((e) => e.level === level && e.arm === arm && e.run === `${arm}-${run}`)
    .sort((a, b) => modelRank(a.model) - modelRank(b.model) || a.model.localeCompare(b.model));
}

// A file that loads on demand: calls and transcripts are only read one entry at a time.
export type Loader = () => Promise<string>;

// The loaders in `files` that belong to one entry, by path.
export function loadersFor(files: Record<string, Loader>, key: string, keyOf: (path: string) => EntryKey | undefined): [string, Loader][] {
  return Object.entries(files).filter(([path]) => {
    const k = keyOf(path);
    return k !== undefined && keyString(k) === key;
  });
}

// Every call file in the gallery, loaded on demand.
export function callLoaders(): Record<string, Loader> {
  return import.meta.glob('../../gallery/*/*/*/calls/*.json', { query: '?raw', import: 'default' }) as Record<string, Loader>;
}

// Every transcript in the gallery, loaded on demand.
export function transcriptLoaders(): Record<string, Loader> {
  return import.meta.glob('../../gallery/*/*/*/transcript.jsonl', { query: '?raw', import: 'default' }) as Record<string, Loader>;
}

const TRANSCRIPT = /gallery\/([^/]+)\/([^/]+)\/([^/]+)\/transcript\.jsonl$/;

// The entry a transcript path belongs to.
export function transcriptKey(path: string): EntryKey | undefined {
  return keyOf(TRANSCRIPT.exec(path));
}

// The whole committed gallery: submissions and summaries, bundled at build time.
export function galleryEntries(): Entry[] {
  const submissions = import.meta.glob('../../gallery/*/*/*/submission.json', {
    query: '?raw',
    import: 'default',
    eager: true,
  }) as Record<string, string>;
  const summaries = import.meta.glob('../../gallery/*/*/*/summary.json', {
    query: '?raw',
    import: 'default',
    eager: true,
  }) as Record<string, string>;
  return indexGallery(submissions, summaries);
}
