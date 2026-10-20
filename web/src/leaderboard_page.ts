// The leaderboard page: one row per model, ranked as `clattertrap leaderboard` ranks them, with
// the counting rules in one line under the table. A click on a column heading sorts by it, and a
// second click turns the order round.

import { band, footer, navLinks, siteCounts } from './chrome';
import { el } from './dom';
import { galleryEntries, gallerySummaries, misses, modelLabel } from './gallery';
import { fixtures, rulesVersionOf, tierOf } from './levels';
import { countLine, leaderboardRows, sortRows, type ArmCount, type Row, type SortKey } from './leaderboard';

const COLUMNS: [string, SortKey][] = [
  ['model', 'model'],
  ['blueprint', 'blueprint'],
  ['workshop', 'workshop'],
  ['hard blueprint', 'hardBlueprint'],
  ['hard workshop', 'hardWorkshop'],
  ['mean USD', 'meanUsd'],
  ['runs', 'runs'],
];

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

class LeaderboardPage {
  private key: SortKey = 'blueprint';
  private flipped = false;
  private readonly headings = new Map<SortKey, HTMLTableCellElement>();
  private readonly body = el('tbody');

  constructor(
    private readonly table: HTMLTableElement,
    private readonly rows: Row[],
  ) {
    const head = el('tr');
    for (const [name, key] of COLUMNS) {
      const th = el('th');
      const button = el('button', name);
      button.addEventListener('click', () => this.sortBy(key));
      th.append(button);
      this.headings.set(key, th);
      head.append(th);
    }
    const thead = el('thead');
    thead.append(head);
    table.append(thead, this.body);
    this.render();
  }

  private sortBy(key: SortKey): void {
    this.flipped = key === this.key && !this.flipped;
    this.key = key;
    this.render();
  }

  private render(): void {
    for (const [key, th] of this.headings) {
      th.classList.toggle('sorted', key === this.key);
      th.dataset.order = key === this.key ? (this.flipped ? 'flipped' : 'natural') : '';
    }
    this.table.dataset.sort = this.key;
    this.body.replaceChildren(...sortRows(this.rows, this.key, this.flipped).map(row));
  }
}

function page(app: HTMLElement): void {
  const levels = fixtures();
  const tiers = new Map(levels.map((f) => [f.id, tierOf(f.level)]));
  const entries = galleryEntries();
  const rows = leaderboardRows(entries, misses(entries, gallerySummaries()), tiers);
  const nav = navLinks([
    ['home', 'index.html'],
    ['play', 'play.html'],
    ['arena', 'arena.html'],
    ['gallery', 'gallery.html'],
    ['level', 'level.html'],
  ]);
  const counts = siteCounts(entries, levels);
  const t = el('table');
  new LeaderboardPage(t, rows);
  const runs = rows.reduce((n, r) => n + r.runs, 0);
  const version = levels[0] === undefined ? 0 : rulesVersionOf(levels[0].level);
  const rules = el(
    'p',
    `${countLine(runs, entries.length, rows.length, version, tiers.size)} ` +
      `Click a heading to sort by it, and again to turn the order round. ` +
      `Every row rests on stored outcomes that clattertrap verify re-simulates.`,
  );
  app.replaceChildren(band('Clattertrap leaderboard', nav, counts), t, rules, footer(counts));
}

const app = document.getElementById('app');
if (app) {
  page(app);
}
