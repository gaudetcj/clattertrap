// The level page: pick a level or a gallery entry, play its reference solution or an opened scene,
// hear it, and read the verdict.

import type { PanelChrome } from './arena_view';
import type { Outcome } from './decode';
import { el } from './dom';
import { galleryEntries, modelLabel, type Entry } from './gallery';
import { fixtures, goalKindOf, rulesVersionOf, titleOf } from './levels';
import { loadWasm } from './replay';
import { Player } from './player';
import { param, search } from './query';
import { stampText } from './stamp';
import { parseOpened } from './submission';

// What a panel shows about the scene on it; the name falls back to the level's title.
interface Shown {
  label: string;
  name?: string;
  prediction?: string;
  outcome?: Outcome;
}

// `seesaw-launch  OPUS 5.5  blueprint-01  RANG 0.7 s`, for the picker.
function entryLabel(entry: Entry, goalOf: (levelId: string) => ReturnType<typeof goalKindOf>): string {
  const goal = goalOf(entry.level);
  const stamp = goal === undefined ? '' : `  ${stampText(goal, entry.outcome)}`;
  return `${entry.level}  ${modelLabel(entry.model)}  ${entry.run}${stamp}`;
}

async function page(app: HTMLElement): Promise<void> {
  const wasm = await loadWasm();
  const levels = fixtures();
  const entries = galleryEntries();
  const goalOf = (id: string) => {
    const f = levels.find((x) => x.id === id);
    return f ? goalKindOf(f.level) : undefined;
  };
  const select = el('select');
  for (const f of levels) {
    const option = el('option', f.id);
    option.value = f.id;
    select.append(option);
  }
  const picker = el('select', undefined, 'runs');
  picker.append(el('option', `gallery (${entries.length})`));
  for (const entry of entries) {
    const option = el('option', entryLabel(entry, goalOf));
    option.value = entry.key;
    picker.append(option);
  }
  const playButton = el('button', 'Play');
  const restartButton = el('button', 'Restart');
  const muteButton = el('button', 'Mute');
  const openInput = el('input');
  openInput.type = 'file';
  openInput.accept = '.json,application/json';
  const openLabel = el('label', 'Open scene ');
  openLabel.append(openInput);
  const controls = el('div', undefined, 'controls');
  controls.append(select, playButton, restartButton, muteButton, openLabel);
  if (entries.length > 0) {
    controls.append(picker);
  }
  const canvas = el('canvas');
  const status = el('p', 'loading', 'status');
  const nav = el('p', undefined, 'nav');
  const arenaLink = el('a', 'arena');
  const galleryLink = el('a', 'gallery');
  galleryLink.href = 'gallery.html';
  nav.append(arenaLink, ' ', galleryLink);
  app.replaceChildren(el('h1', 'Clattertrap'), nav, controls, canvas, status);

  const player = new Player(canvas, wasm.dt_s, (text) => {
    status.textContent = text;
    playButton.textContent = player.isPlaying() ? 'Pause' : 'Play';
  });
  // Loads a scene onto its level; the reference solution when no scene is given.
  const show = (levelId: string, sceneJson: string | null, shown: Shown): void => {
    const f = levels.find((x) => x.id === levelId);
    if (!f) {
      status.textContent = `no level ${levelId}`;
      return;
    }
    if (rulesVersionOf(f.level) !== wasm.rules_version) {
      status.textContent = `${f.id} was written for rules v${rulesVersionOf(f.level)}; this build runs v${wasm.rules_version}`;
      return;
    }
    select.value = f.id;
    arenaLink.href = `arena.html${search({ level: f.id })}`;
    const chrome: PanelChrome = { label: shown.label, name: shown.name ?? titleOf(f.level), prediction: shown.prediction ?? '' };
    try {
      player.load(f.level, sceneJson ?? f.reference, chrome, shown.outcome ?? null);
    } catch (e) {
      status.textContent = e instanceof Error ? e.message : String(e);
    }
  };
  // Keeps the picker and the address bar naming the open entry, so a replay can be linked and reloaded.
  const address = (key: string | undefined): void => {
    picker.selectedIndex = key === undefined ? 0 : entries.findIndex((e) => e.key === key) + 1;
    history.replaceState(null, '', key === undefined ? location.pathname : search({ entry: key }));
  };
  const load = (): void => {
    address(undefined);
    show(select.value, null, { label: 'REFERENCE' });
  };
  const openText = (text: string, label: string): void => {
    const opened = parseOpened(text);
    show(opened.levelId, opened.sceneJson, { ...opened, label });
  };
  const open = async (file: File): Promise<void> => {
    try {
      address(undefined);
      openText(await file.text(), file.name);
    } catch (e) {
      status.textContent = `${file.name}: ${e instanceof Error ? e.message : String(e)}`;
    }
  };
  const openEntry = (key: string): void => {
    const entry = entries.find((e) => e.key === key);
    if (!entry) {
      status.textContent = `no gallery entry ${key}`;
      return;
    }
    address(key);
    show(entry.level, entry.sceneJson, { label: modelLabel(entry.model), name: entry.name, prediction: entry.prediction, outcome: entry.outcome });
  };
  picker.addEventListener('change', () => {
    if (picker.selectedIndex > 0) {
      openEntry(picker.value);
    }
  });
  openInput.addEventListener('change', () => {
    const file = openInput.files?.[0];
    if (file) {
      void open(file);
    }
  });
  app.addEventListener('dragover', (e) => e.preventDefault());
  app.addEventListener('drop', (e) => {
    e.preventDefault();
    const file = e.dataTransfer?.files[0];
    if (file) {
      void open(file);
    }
  });
  select.addEventListener('change', load);
  playButton.addEventListener('click', () => player.toggle());
  restartButton.addEventListener('click', () => player.restart());
  let muted = false;
  muteButton.addEventListener('click', () => {
    muted = !muted;
    player.setMuted(muted);
    muteButton.textContent = muted ? 'Unmute' : 'Mute';
  });
  const linked = param(location.search, 'entry');
  if (linked === undefined) {
    load();
  } else {
    openEntry(linked);
  }
}

const app = document.getElementById('app');
if (app) {
  page(app).catch((e: unknown) => {
    app.textContent = e instanceof Error ? e.message : String(e);
  });
}
