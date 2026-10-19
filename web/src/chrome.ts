// The chrome every page shares, in the machines' own look: the title band with its nav and
// marquee, and the footer with the machine counter and the badges.

import { el } from './dom';
import type { Entry } from './gallery';
import { rulesVersionOf, type Fixture } from './levels';

export interface Counts {
  machines: number;
  models: number;
  levels: number;
  rulesVersion: number;
}

// The digits of the counter: zero-padded, saturating at all nines.
export function odometer(n: number, digits: number): string[] {
  const max = 10 ** digits - 1;
  return String(Math.min(Math.max(0, Math.floor(n)), max))
    .padStart(digits, '0')
    .split('');
}

export function marqueeLine(c: Counts): string {
  return [
    `${c.machines} MACHINES BUILT BY ${c.models} MODELS ON ${c.levels} LEVELS`,
    `RULES V${c.rulesVersion}`,
    'EVERY REPLAY RE-SIMULATED IN YOUR BROWSER',
    'SAME SCENE, SAME RESULT, ON EVERY MACHINE',
    'BEST VIEWED AT 1 CM PER PIXEL',
  ].join('   *   ');
}

export function siteCounts(entries: Entry[], levels: Fixture[]): Counts {
  const first = levels[0];
  return {
    machines: entries.length,
    models: new Set(entries.map((e) => e.model)).size,
    levels: levels.length,
    rulesVersion: first === undefined ? 0 : rulesVersionOf(first.level),
  };
}

// A nav paragraph of links, `[text, href]` each.
export function navLinks(links: [string, string][]): HTMLElement {
  const nav = el('p', undefined, 'nav');
  for (const [text, href] of links) {
    const a = el('a', text);
    a.href = href;
    nav.append(a, ' ');
  }
  return nav;
}

// The title band: the heading, the nav beside it, and the marquee under both.
export function band(title: string, nav: HTMLElement, counts: Counts): HTMLElement {
  const header = el('header', undefined, 'band');
  const row = el('div', undefined, 'band-row');
  row.append(el('h1', title), nav);
  const marquee = el('p', undefined, 'marquee');
  marquee.append(el('span', marqueeLine(counts)));
  header.append(row, marquee);
  return header;
}

// The footer: the machine counter and the badges.
export function footer(counts: Counts): HTMLElement {
  const foot = el('footer', undefined, 'foot');
  const counter = el('p', undefined, 'counter');
  counter.append(el('span', 'MACHINES BUILT'));
  for (const digit of odometer(counts.machines, 6)) {
    counter.append(el('b', digit));
  }
  const badges = el('p', undefined, 'badges');
  const labels = [
    `RULES V${counts.rulesVersion}`,
    `${counts.levels} LEVELS`,
    'WASM INSIDE',
    '100% DETERMINISTIC',
    'RAPIER 2D',
    'NO SAMPLES, ALL SYNTH',
  ];
  for (const label of labels) {
    badges.append(el('span', label, 'badge'));
  }
  foot.append(counter, badges);
  return foot;
}
