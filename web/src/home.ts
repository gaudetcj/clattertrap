// The home page: the write-up in index.html gets the title band, the footer, the numbers in its
// fast facts, and the top of the leaderboard in its sidebar. Links from before it existed are sent
// on to the page they meant.

import { band, footer, navLinks, siteCounts, type Counts } from './chrome';
import { el } from './dom';
import { forwardFor } from './forward';
import { galleryEntries, gallerySummaries, misses, modelLabel, type Entry } from './gallery';
import { leaderboardRows, type ArmCount, type Row } from './leaderboard';
import { fixtures, tierOf, type Fixture } from './levels';

// How many rows the sidebar shows.
const TOP = 3;

function fillFacts(app: HTMLElement, counts: Counts): void {
  const facts: Record<string, string> = {
    machines: String(counts.machines),
    models: String(counts.models),
    levels: String(counts.levels),
    rules: `v${counts.rulesVersion}`,
  };
  for (const dd of app.querySelectorAll<HTMLElement>('[data-fact]')) {
    const value = facts[dd.dataset['fact'] ?? ''];
    if (value !== undefined) {
      dd.textContent = value;
    }
  }
}

function fraction(c: ArmCount): string {
  return `${c.solved}/${c.attempted}`;
}

function boardRow(r: Row): HTMLTableRowElement {
  const tr = el('tr');
  tr.append(el('td', modelLabel(r.model)), el('td', fraction(r.blueprint)), el('td', fraction(r.workshop)));
  return tr;
}

// The top rows of the leaderboard, with a link to the whole table.
function fillBoard(box: HTMLElement, entries: Entry[], levels: Fixture[]): void {
  const tiers = new Map(levels.map((f) => [f.id, tierOf(f.level)]));
  const rows = leaderboardRows(entries, misses(entries, gallerySummaries()), tiers).slice(0, TOP);
  const table = el('table', undefined, 'board');
  const head = el('tr');
  for (const name of ['model', 'blueprint', 'workshop']) {
    head.append(el('th', name));
  }
  table.append(head, ...rows.map(boardRow));
  const more = el('p', undefined, 'muted');
  const link = el('a', 'the whole leaderboard');
  link.href = 'leaderboard.html';
  more.append('Levels solved over attempted; ', link, '.');
  const heading = box.querySelector('h3');
  box.replaceChildren(...(heading ? [heading] : []), table, more);
}

function page(app: HTMLElement): void {
  const onward = forwardFor(location.search);
  if (onward !== undefined) {
    location.replace(onward);
    return;
  }
  const entries = galleryEntries();
  const levels = fixtures();
  const counts = siteCounts(entries, levels);
  const nav = navLinks([
    ['arena', 'arena.html'],
    ['gallery', 'gallery.html'],
    ['leaderboard', 'leaderboard.html'],
    ['level', 'level.html'],
  ]);
  app.prepend(band('Clattertrap', nav, counts));
  fillFacts(app, counts);
  const board = app.querySelector<HTMLElement>('#board');
  if (board) {
    fillBoard(board, entries, levels);
  }
  app.append(footer(counts));
}

const app = document.getElementById('app');
if (app) {
  page(app);
}
