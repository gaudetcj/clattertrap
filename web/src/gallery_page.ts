// The gallery page: every entry, one table per level, with the model, the run, the machine name,
// the stored verdict, the cost, and links to replay it alone, in the arena, or as a reel.

import { el } from './dom';
import { byLevel, galleryEntries, gallerySummaries, misses, modelLabel, type Entry } from './gallery';
import { fixtures, goalKindOf } from './levels';
import { search } from './query';
import { stampText } from './stamp';

function link(text: string, href: string): HTMLAnchorElement {
  const a = el('a', text);
  a.href = href;
  return a;
}

function cell(content: string | Node): HTMLTableCellElement {
  const td = el('td');
  td.append(content);
  return td;
}

function row(entry: Entry, goal: ReturnType<typeof goalKindOf>): HTMLTableRowElement {
  const tr = el('tr');
  const cost = entry.summary?.cost_usd;
  const seconds = entry.summary?.duration_s;
  tr.append(
    cell(modelLabel(entry.model)),
    cell(entry.run),
    cell(link(entry.name, `index.html${search({ entry: entry.key })}`)),
    cell(goal === undefined ? '' : stampText(goal, entry.outcome)),
    cell(cost === undefined || cost === null ? '' : `${cost.toFixed(2)} USD`),
    cell(seconds === undefined ? '' : `${seconds.toFixed(0)} s`),
    cell(link('reel', `reel.html${search({ entry: entry.key })}`)),
  );
  return tr;
}

function table(level: string, entries: Entry[], goal: ReturnType<typeof goalKindOf>): HTMLElement[] {
  const heading = el('h2', level);
  const arms = [...new Set(entries.map((e) => e.arm))].sort();
  const nav = el('p', undefined, 'nav');
  for (const arm of arms) {
    nav.append(link(`arena ${arm}`, `arena.html${search({ level, arm })}`), ' ');
  }
  const t = el('table');
  const head = el('tr');
  for (const name of ['model', 'run', 'machine', 'result', 'cost', 'time', '']) {
    head.append(el('th', name));
  }
  t.append(head, ...entries.map((e) => row(e, goal)));
  return [heading, nav, t];
}

function page(app: HTMLElement): void {
  const levels = fixtures();
  const entries = galleryEntries();
  const nav = el('p', undefined, 'nav');
  nav.append(link('level', 'index.html'), ' ', link('arena', 'arena.html'));
  app.replaceChildren(el('h1', `Clattertrap gallery: ${entries.length} machines`), nav);
  for (const [level, mine] of byLevel(entries)) {
    const f = levels.find((x) => x.id === level);
    app.append(...table(level, mine, f ? goalKindOf(f.level) : undefined));
  }
  const missed = misses(entries, gallerySummaries());
  if (missed.length > 0) {
    app.append(el('h2', 'counted runs without a submission'));
    const list = el('ul');
    for (const m of missed) {
      list.append(el('li', `${m.level}  ${modelLabel(m.model)}  ${m.run}  ${m.status}`));
    }
    app.append(list);
  }
}

const app = document.getElementById('app');
if (app) {
  page(app);
}
