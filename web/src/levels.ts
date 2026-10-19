// The levels bundled into every page, with their reference solutions, and the few fields a page
// reads from a level's JSON.

import type { GoalKind } from './decode';
import { GOAL_KINDS } from './decode';

export interface Fixture {
  id: string;
  level: string;
  reference: string;
}

// Every level in levels/ with its reference solution, bundled as raw JSON.
export function fixtures(): Fixture[] {
  const files = import.meta.glob('../../levels/*/{level,reference}.json', {
    query: '?raw',
    import: 'default',
    eager: true,
  }) as Record<string, string>;
  return indexFixtures(files);
}

export function indexFixtures(files: Record<string, string>): Fixture[] {
  const byId = new Map<string, Partial<Fixture>>();
  for (const [path, text] of Object.entries(files)) {
    const m = /levels\/([^/]+)\/(level|reference)\.json$/.exec(path);
    const id = m?.[1];
    if (!m || id === undefined) {
      continue;
    }
    const entry = byId.get(id) ?? { id };
    if (m[2] === 'level') {
      entry.level = text;
    } else {
      entry.reference = text;
    }
    byId.set(id, entry);
  }
  const out: Fixture[] = [];
  for (const f of byId.values()) {
    if (f.id !== undefined && f.level !== undefined && f.reference !== undefined) {
      out.push({ id: f.id, level: f.level, reference: f.reference });
    }
  }
  return out.sort((a, b) => a.id.localeCompare(b.id));
}

function field(levelJson: string, name: string): unknown {
  const parsed: unknown = JSON.parse(levelJson);
  return typeof parsed === 'object' && parsed !== null ? (parsed as Record<string, unknown>)[name] : undefined;
}

export function rulesVersionOf(levelJson: string): number {
  const v = field(levelJson, 'rules_version');
  return typeof v === 'number' ? v : -1;
}

export function titleOf(levelJson: string): string {
  const v = field(levelJson, 'title');
  return typeof v === 'string' ? v : '';
}

export function goalKindOf(levelJson: string): GoalKind | undefined {
  const goal = field(levelJson, 'goal');
  const kind = typeof goal === 'object' && goal !== null ? (goal as { kind?: unknown }).kind : undefined;
  return GOAL_KINDS.find((k) => k === kind);
}

export function tierOf(levelJson: string): number {
  const v = field(levelJson, 'tier');
  return typeof v === 'number' ? v : 0;
}
