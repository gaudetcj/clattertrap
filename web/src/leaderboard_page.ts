// The leaderboard page: one row per model, ranked as `clattertrap leaderboard` ranks them, with
// the counting rules in one line under the table.

import { band, footer, navLinks, siteCounts } from './chrome';
import { el } from './dom';
import { galleryEntries, gallerySummaries, misses, modelLabel } from './gallery';
import { fixtures, rulesVersionOf, tierOf } from './levels';
import { HARD_TIER, leaderboardRows, type ArmCount, type Row } from './leaderboard';

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

function fraction(c: ArmCount): string {
  return `${c.solved}/${c.attempted}`;
}

function hard(c: ArmCount): string {
  return `${c.hardSolved}/${c.hardAttempted}`;
}

function row(r: Row): HTMLTableRowElement {
  const tr = el('tr');
  tr.append(
    cell(link(modelLabel(r.model), 'gallery.html')),
    cell(fraction(r.blueprint)),
    cell(fraction(r.workshop)),
    cell(hard(r.blueprint)),
    cell(hard(r.workshop)),
    cell(r.meanUsd.toFixed(2)),
    cell(String(r.runs)),
  );
  return tr;
}

function page(app: HTMLElement): void {
  const levels = fixtures();
  const tiers = new Map(levels.map((f) => [f.id, tierOf(f.level)]));
  const entries = galleryEntries();
  const rows = leaderboardRows(entries, misses(entries, gallerySummaries()), tiers);
  const nav = navLinks([
    ['arena', 'index.html'],
    ['gallery', 'gallery.html'],
    ['level', 'level.html'],
  ]);
  const counts = siteCounts(entries, levels);
  const t = el('table');
  const head = el('tr');
  for (const name of ['model', 'blueprint', 'workshop', 'hard blueprint', 'hard workshop', 'mean USD', 'runs']) {
    head.append(el('th', name));
  }
  t.append(head, ...rows.map(row));
  const runs = rows.reduce((n, r) => n + r.runs, 0);
  const version = levels[0] === undefined ? 0 : rulesVersionOf(levels[0].level);
  const rules = el(
    'p',
    `${runs} counted runs by ${rows.length} models on rules version ${version}. ` +
      `A level counts once however many runs solved it, over the levels the model attempted in that arm, of ${tiers.size}; ` +
      `hard levels are tiers ${HARD_TIER} and 5. Every row rests on stored outcomes that clattertrap verify re-simulates.`,
  );
  app.replaceChildren(band('Clattertrap leaderboard', nav, counts), t, rules, footer(counts));
}

const app = document.getElementById('app');
if (app) {
  page(app);
}
