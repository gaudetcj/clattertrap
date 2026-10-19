// The arena page: one level, one arm, one panel per model, all released at once. The verdicts are
// mirrored into the DOM so a test can read them without touching the canvas.

import { defaultLineup, lanesFor, LINEUPS, lineupNames, nextLevelId, pickFrom, pickModels, presentModels, type LineupKind, type Pick } from './arena_lanes';
import { Conductor } from './conductor';
import { band, footer, navLinks, siteCounts } from './chrome';
import { el } from './dom';
import { galleryEntries, gallerySummaries, misses, type Entry, type Miss } from './gallery';
import { leaderboardRows } from './leaderboard';
import { fixtures, rulesVersionOf, tierOf, type Fixture } from './levels';
import { param, search } from './query';
import { installRecorder } from './record';
import { loadWasm } from './replay';

const ARMS = ['blueprint', 'workshop'];

// The lineup dropdown's extra entry for the custom list a `?models=` link carries.
const CUSTOM = 'custom';

// The models in leaderboard order, which every lineup follows.
function rankedModels(entries: Entry[], missed: Miss[], levels: Fixture[]): string[] {
  const tiers = new Map(levels.map((f) => [f.id, tierOf(f.level)]));
  return leaderboardRows(entries, missed, tiers).map((r) => r.model);
}

// A select with its name in front, so a visitor knows it picks something.
function labelled(name: string, select: HTMLSelectElement): HTMLLabelElement {
  const label = el('label', `${name} `, 'pick');
  label.append(select);
  return label;
}

class ArenaPage {
  readonly conductor: Conductor;
  private readonly status = el('p', 'loading', 'status');
  private readonly levelSelect = el('select');
  private readonly armSelect = el('select');
  private readonly lineupSelect = el('select');
  private readonly playButton = el('button', 'Play');
  private readonly canvas = el('canvas');
  private readonly levelIds: string[];
  private readonly ranked: string[];
  private muted = false;

  constructor(
    app: HTMLElement,
    private readonly dt_s: number,
    private readonly rules_version: number,
    private readonly levels: Fixture[],
    private readonly entries: Entry[],
    private readonly missed: Miss[],
    private pick: Pick,
    private readonly record: boolean,
  ) {
    const withEntries = levels.filter((f) => entries.some((e) => e.level === f.id));
    this.levelIds = withEntries.map((f) => f.id);
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
    for (const kind of LINEUPS) {
      const option = el('option', kind);
      option.value = kind;
      this.lineupSelect.append(option);
    }
    if (pick.models) {
      const option = el('option', CUSTOM);
      option.value = CUSTOM;
      this.lineupSelect.append(option);
    }
    this.ranked = rankedModels(entries, missed, levels);
    const restartButton = el('button', 'Restart');
    const muteButton = el('button', 'Mute');
    const nextButton = el('button', 'Next puzzle');
    const controls = el('div', undefined, 'controls');
    controls.append(
      labelled('puzzle', this.levelSelect),
      labelled('arm', this.armSelect),
      labelled('lineup', this.lineupSelect),
      nextButton,
      this.playButton,
      restartButton,
      muteButton,
    );
    const hint = el('p', 'Pick a puzzle, an arm, and a lineup. Blueprint: no test runs, the model submitted blind. Workshop: up to 8 test runs first.', 'hint');
    const nav = navLinks([
      ['home', 'index.html'],
      ['play', 'play.html'],
      ['gallery', 'gallery.html'],
      ['leaderboard', 'leaderboard.html'],
      ['level', 'level.html'],
    ]);
    const counts = siteCounts(entries, levels);
    app.replaceChildren(band('Clattertrap arena', nav, counts), controls, hint, this.canvas, this.status, footer(counts));
    this.conductor = new Conductor(this.canvas, this.dt_s, !record, (c) => this.changed(c));
    this.levelSelect.addEventListener('change', () => this.open(this.repick({ level: this.levelSelect.value })));
    this.armSelect.addEventListener('change', () => this.open(this.repick({ arm: this.armSelect.value })));
    this.lineupSelect.addEventListener('change', () => {
      const kind = this.lineupSelect.value;
      const next: Pick = { level: this.pick.level, arm: this.pick.arm };
      if (kind !== CUSTOM && LINEUPS.includes(kind as LineupKind)) {
        next.lineup = kind as LineupKind;
      }
      this.open(next);
    });
    this.playButton.addEventListener('click', () => this.conductor.toggle());
    nextButton.addEventListener('click', () => this.open(this.repick({ level: nextLevelId(this.levelIds, this.pick.level) })));
    restartButton.addEventListener('click', () => this.conductor.restart());
    muteButton.addEventListener('click', () => {
      this.muted = !this.muted;
      this.conductor.setMuted(this.muted);
      muteButton.textContent = this.muted ? 'Unmute' : 'Mute';
    });
    window.addEventListener('resize', () => this.conductor.fit());
    this.open(pick);
  }

  // The current pick with a level or arm changed, keeping the lineup and dropping the run number.
  private repick(change: { level?: string; arm?: string }): Pick {
    const next: Pick = { level: change.level ?? this.pick.level, arm: change.arm ?? this.pick.arm };
    if (this.pick.lineup) {
      next.lineup = this.pick.lineup;
    }
    if (this.pick.models) {
      next.models = this.pick.models;
    }
    return next;
  }

  // Loads the lineup for a level and arm and names it in the address bar.
  open(pick: Pick): void {
    this.pick = pick;
    this.levelSelect.value = pick.level;
    this.armSelect.value = pick.arm;
    const present = presentModels(this.entries, this.missed, pick);
    const names = lineupNames(this.ranked, present);
    for (const option of this.lineupSelect.options) {
      if (option.value !== CUSTOM) {
        option.textContent = names[option.value as LineupKind];
      }
    }
    this.lineupSelect.value = pick.models ? CUSTOM : (pick.lineup ?? defaultLineup(present.length));
    const address: Record<string, string> = { level: pick.level, arm: pick.arm };
    if (pick.lineup) {
      address['lineup'] = pick.lineup;
    }
    if (pick.models) {
      address['models'] = pick.models.join(',');
    }
    history.replaceState(null, '', search(address));
    const level = this.levels.find((f) => f.id === pick.level);
    if (level && rulesVersionOf(level.level) !== this.rules_version) {
      this.status.textContent = `${pick.level} was written for rules v${rulesVersionOf(level.level)}; this build runs v${this.rules_version}`;
      return;
    }
    const lanes = lanesFor(this.entries, this.missed, this.levels, pick, this.ranked);
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
      // A lane without a machine never moves, so it has no sound.
      installRecorder(this.canvas, this.conductor, lanes.filter((l) => l.missing === undefined), this.dt_s);
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
    this.status.dataset['settled'] = String(c.settled());
    const models = pickModels(this.entries, this.missed, this.pick, this.ranked);
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
  const wasm = await loadWasm();
  const record = param(query, 'record') !== undefined;
  const entries = galleryEntries();
  const arena = new ArenaPage(app, wasm.dt_s, wasm.rules_version, fixtures(), entries, misses(entries, gallerySummaries()), pickFrom(query), record);
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
