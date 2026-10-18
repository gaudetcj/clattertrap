// The level page: pick a level or a pilot run, play its reference solution or an opened scene, hear
// it, and read the verdict.

import type { PanelChrome } from './arena_view';
import type { Outcome } from './decode';
import { el } from './dom';
import { fixtures, goalKindOf, rulesVersionOf, titleOf } from './levels';
import { loadWasm } from './replay';
import { Player } from './player';
import { indexRuns, runFromSearch, searchForRun } from './runs';
import type { Run } from './runs';
import { parseOpened } from './submission';

// Every pilot submission under pilot/runs, when the checkout has any; the gallery replaces this in time.
function pilotRuns(goalOf: (levelId: string) => ReturnType<typeof goalKindOf>): Run[] {
  const files = import.meta.glob('../../pilot/runs/*/*/*/submission.json', {
    query: '?raw',
    import: 'default',
    eager: true,
  }) as Record<string, string>;
  return indexRuns(files, goalOf);
}

// What a panel shows about the scene on it; the name falls back to the level's title.
interface Shown {
  label: string;
  name?: string;
  prediction?: string;
  outcome?: Outcome;
}

async function page(app: HTMLElement): Promise<void> {
  const wasm = await loadWasm();
  const levels = fixtures();
  const runs = pilotRuns((id) => {
    const f = levels.find((x) => x.id === id);
    return f ? goalKindOf(f.level) : undefined;
  });
  const select = el('select');
  for (const f of levels) {
    const option = el('option', f.id);
    option.value = f.id;
    select.append(option);
  }
  const picker = el('select', undefined, 'runs');
  picker.append(el('option', `pilot runs (${runs.length})`));
  for (const run of runs) {
    const option = el('option', run.label);
    option.value = run.key;
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
  if (runs.length > 0) {
    controls.append(picker);
  }
  const canvas = el('canvas');
  const status = el('p', 'loading', 'status');
  app.replaceChildren(el('h1', 'Clattertrap'), controls, canvas, status);

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
    const chrome: PanelChrome = { label: shown.label, name: shown.name ?? titleOf(f.level), prediction: shown.prediction ?? '' };
    try {
      player.load(f.level, sceneJson ?? f.reference, chrome, shown.outcome ?? null);
    } catch (e) {
      status.textContent = e instanceof Error ? e.message : String(e);
    }
  };
  // Keeps the picker and the address bar naming the open run, so a replay can be linked and reloaded.
  const address = (key: string | undefined): void => {
    picker.selectedIndex = key === undefined ? 0 : runs.findIndex((r) => r.key === key) + 1;
    history.replaceState(null, '', key === undefined ? location.pathname : searchForRun(key));
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
  const openRun = (key: string): void => {
    const run = runs.find((r) => r.key === key);
    if (!run) {
      status.textContent = `no pilot run ${key}`;
      return;
    }
    address(key);
    try {
      openText(run.text, key.split('/')[0] ?? key);
    } catch (e) {
      status.textContent = `${key}: ${e instanceof Error ? e.message : String(e)}`;
    }
  };
  picker.addEventListener('change', () => {
    if (picker.selectedIndex > 0) {
      openRun(picker.value);
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
  const linked = runFromSearch(location.search);
  if (linked === undefined) {
    load();
  } else {
    openRun(linked);
  }
}

const app = document.getElementById('app');
if (app) {
  page(app).catch((e: unknown) => {
    app.textContent = e instanceof Error ? e.message : String(e);
  });
}
