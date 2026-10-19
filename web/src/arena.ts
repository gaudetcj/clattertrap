// The arena page: one level, one arm, one panel per model, all released at once. The verdicts are
// mirrored into the DOM so a test can read them without touching the canvas.

import { lanesFor, pickFrom, type Pick } from './arena_lanes';
import { Conductor } from './conductor';
import { band, footer, navLinks, siteCounts } from './chrome';
import { el } from './dom';
import { galleryEntries, lineup, type Entry } from './gallery';
import { fixtures, rulesVersionOf, type Fixture } from './levels';
import { param, search } from './query';
import { installRecorder } from './record';
import { loadWasm } from './replay';

const ARMS = ['blueprint', 'workshop'];

class ArenaPage {
  readonly conductor: Conductor;
  private readonly status = el('p', 'loading', 'status');
  private readonly levelSelect = el('select');
  private readonly armSelect = el('select');
  private readonly playButton = el('button', 'Play');
  private readonly canvas = el('canvas');
  private muted = false;

  constructor(
    app: HTMLElement,
    private readonly dt_s: number,
    private readonly rules_version: number,
    private readonly levels: Fixture[],
    private readonly entries: Entry[],
    private pick: Pick,
    private readonly record: boolean,
  ) {
    const withEntries = levels.filter((f) => entries.some((e) => e.level === f.id));
    for (const f of withEntries) {
      const option = el('option', f.id);
      option.value = f.id;
      this.levelSelect.append(option);
    }
    for (const arm of ARMS) {
      const option = el('option', arm);
      option.value = arm;
      this.armSelect.append(option);
    }
    const restartButton = el('button', 'Restart');
    const muteButton = el('button', 'Mute');
    const controls = el('div', undefined, 'controls');
    controls.append(this.levelSelect, this.armSelect, this.playButton, restartButton, muteButton);
    const nav = navLinks([
      ['gallery', 'gallery.html'],
      ['leaderboard', 'leaderboard.html'],
      ['level', 'level.html'],
    ]);
    const counts = siteCounts(entries, levels);
    app.replaceChildren(band('Clattertrap', nav, counts), controls, this.canvas, this.status, footer(counts));
    this.conductor = new Conductor(this.canvas, this.dt_s, !record, (c) => this.changed(c));
    this.levelSelect.addEventListener('change', () => this.open({ level: this.levelSelect.value, arm: this.pick.arm }));
    this.armSelect.addEventListener('change', () => this.open({ level: this.pick.level, arm: this.armSelect.value }));
    this.playButton.addEventListener('click', () => this.conductor.toggle());
    restartButton.addEventListener('click', () => this.conductor.restart());
    muteButton.addEventListener('click', () => {
      this.muted = !this.muted;
      this.conductor.setMuted(this.muted);
      muteButton.textContent = this.muted ? 'Unmute' : 'Mute';
    });
    window.addEventListener('resize', () => this.conductor.fit());
    this.open(pick);
  }

  // Loads the lineup for a level and arm and names it in the address bar.
  open(pick: Pick): void {
    this.pick = pick;
    this.levelSelect.value = pick.level;
    this.armSelect.value = pick.arm;
    history.replaceState(null, '', search({ level: pick.level, arm: pick.arm }));
    const level = this.levels.find((f) => f.id === pick.level);
    if (level && rulesVersionOf(level.level) !== this.rules_version) {
      this.status.textContent = `${pick.level} was written for rules v${rulesVersionOf(level.level)}; this build runs v${this.rules_version}`;
      return;
    }
    const lanes = lanesFor(this.entries, this.levels, pick);
    if (lanes.length === 0) {
      this.status.textContent = `no gallery entries for ${pick.level} ${pick.arm}`;
      return;
    }
    try {
      this.conductor.load(lanes);
    } catch (e) {
      this.status.textContent = e instanceof Error ? e.message : String(e);
      return;
    }
    if (this.record) {
      installRecorder(this.canvas, this.conductor, lanes, this.dt_s);
    }
  }

  // Starts muted, as a page may without a click.
  autoplay(): void {
    this.muted = true;
    this.conductor.setMuted(true);
    this.conductor.play();
  }

  private changed(c: Conductor): void {
    this.playButton.textContent = c.isPlaying() ? 'Pause' : 'Play';
    this.status.dataset['ended'] = String(c.ended());
    const models = lineup(this.entries, this.pick.level, this.pick.arm, this.pick.run).map((e) => e.model);
    const spans = c.verdicts().map((stamp, i) => {
      const span = el('span', stamp?.text ?? '-', 'stamp');
      span.dataset['model'] = models[i] ?? '';
      return span;
    });
    this.status.replaceChildren(el('span', `${c.state()} `), ...spans);
  }
}

async function page(app: HTMLElement): Promise<void> {
  // Read before the page rewrites the address to the level and arm it opened.
  const query = location.search;
  // A machine link from before the arena became the landing page opens on the level page.
  if (param(query, 'entry') !== undefined) {
    location.replace(`level.html${query}`);
    return;
  }
  const wasm = await loadWasm();
  const record = param(query, 'record') !== undefined;
  const arena = new ArenaPage(app, wasm.dt_s, wasm.rules_version, fixtures(), galleryEntries(), pickFrom(query), record);
  // The landing page plays on arrival, muted; a recorder drives the clock itself.
  if (!record) {
    arena.autoplay();
  }
}

const app = document.getElementById('app');
if (app) {
  page(app).catch((e: unknown) => {
    app.textContent = e instanceof Error ? e.message : String(e);
  });
}
