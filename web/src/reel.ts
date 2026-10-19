// The attempt reel page: one model's attempts on one level in order, attempt 1 through the
// submission, each held on its stamp before the next, with the model's notes beside them.

import { Conductor } from './conductor';
import { el } from './dom';
import {
  callKey,
  callLoaders,
  galleryEntries,
  loadersFor,
  modelLabel,
  transcriptKey,
  transcriptLoaders,
  type Entry,
} from './gallery';
import { fixtures, type Fixture } from './levels';
import { param } from './query';
import { notesFrom, parseCall, reelSteps, type ReelStep } from './reel_plan';
import { loadWasm } from './replay';

// How long a finished attempt stays on its stamp before the next one starts.
const HOLD_MS = 2000;

class ReelPage {
  private readonly conductor: Conductor;
  private readonly status = el('p', 'loading', 'status');
  private readonly canvas = el('canvas');
  private readonly notes = el('div', undefined, 'notes');
  private steps: ReelStep[] = [];
  private at = 0;
  private hold: number | null = null;

  constructor(
    app: HTMLElement,
    dt_s: number,
    private readonly level: Fixture,
    private readonly entry: Entry,
  ) {
    const playButton = el('button', 'Play');
    const restartButton = el('button', 'Restart');
    const muteButton = el('button', 'Mute');
    const controls = el('div', undefined, 'controls');
    controls.append(playButton, restartButton, muteButton);
    const nav = el('p', undefined, 'nav');
    const arenaLink = el('a', 'arena');
    arenaLink.href = `arena.html?level=${encodeURIComponent(entry.level)}&arm=${encodeURIComponent(entry.arm)}`;
    const galleryLink = el('a', 'gallery');
    galleryLink.href = 'gallery.html';
    const boardLink = el('a', 'leaderboard');
    boardLink.href = 'leaderboard.html';
    nav.append(arenaLink, ' ', galleryLink, ' ', boardLink);
    app.replaceChildren(el('h1', `${modelLabel(entry.model)} on ${entry.level}, ${entry.run}`), nav, controls, this.canvas, this.status, this.notes);
    let muted = false;
    this.conductor = new Conductor(this.canvas, dt_s, true, (c) => {
      playButton.textContent = c.isPlaying() ? 'Pause' : 'Play';
      this.status.dataset['ended'] = String(c.ended());
      if (c.ended()) {
        this.scheduleNext();
      }
    });
    playButton.addEventListener('click', () => this.conductor.toggle());
    restartButton.addEventListener('click', () => this.show(0, this.conductor.isPlaying()));
    muteButton.addEventListener('click', () => {
      muted = !muted;
      this.conductor.setMuted(muted);
      muteButton.textContent = muted ? 'Unmute' : 'Mute';
    });
    window.addEventListener('resize', () => this.conductor.fit());
  }

  start(steps: ReelStep[], notes: string[]): void {
    this.steps = steps;
    this.notes.replaceChildren(...notes.map((n) => el('p', n)));
    if (steps.length === 0) {
      this.status.textContent = 'nothing to replay';
      return;
    }
    this.show(0, false);
  }

  private show(at: number, play: boolean): void {
    if (this.hold !== null) {
      clearTimeout(this.hold);
      this.hold = null;
    }
    const step = this.steps[at];
    if (!step) {
      return;
    }
    this.at = at;
    this.status.dataset['step'] = String(at + 1);
    this.status.textContent = `${at + 1} of ${this.steps.length}: ${step.label.toLowerCase()}`;
    try {
      this.conductor.load([
        {
          chrome: { label: `${modelLabel(this.entry.model)}  ${step.label}`, name: step.name, prediction: step.prediction },
          levelJson: this.level.level,
          sceneJson: step.sceneJson,
          stored: step.stored,
        },
      ]);
    } catch (e) {
      this.status.textContent = e instanceof Error ? e.message : String(e);
      return;
    }
    if (play) {
      this.conductor.play();
    }
  }

  // After the hold, the next attempt starts on its own; the last one stays.
  private scheduleNext(): void {
    if (this.hold !== null || this.at + 1 >= this.steps.length) {
      return;
    }
    this.hold = window.setTimeout(() => {
      this.hold = null;
      this.show(this.at + 1, true);
    }, HOLD_MS);
  }
}

async function page(app: HTMLElement): Promise<void> {
  const key = param(location.search, 'entry');
  const entries = galleryEntries();
  const entry = entries.find((e) => e.key === key);
  const level = entry ? fixtures().find((f) => f.id === entry.level) : undefined;
  if (!entry || !level) {
    app.textContent = key === undefined ? 'open ?entry=<level>/<model>/<arm>-NN' : `no gallery entry ${key}`;
    return;
  }
  const wasm = await loadWasm();
  const reel = new ReelPage(app, wasm.dt_s, level, entry);
  const calls = await Promise.all(loadersFor(callLoaders(), entry.key, (p) => callKey(p)?.key).map(([, load]) => load()));
  const transcripts = await Promise.all(loadersFor(transcriptLoaders(), entry.key, transcriptKey).map(([, load]) => load()));
  reel.start(
    reelSteps(calls.map(parseCall).flatMap((c) => (c ? [c] : [])), entry),
    transcripts.flatMap(notesFrom),
  );
}

const app = document.getElementById('app');
if (app) {
  page(app).catch((e: unknown) => {
    app.textContent = e instanceof Error ? e.message : String(e);
  });
}
