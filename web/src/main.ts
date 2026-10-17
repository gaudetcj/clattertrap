// The level page: pick a fixture level, play its reference solution, hear it, and read the verdict.

import { loadWasm } from './replay';
import { Player } from './player';

interface Fixture {
  id: string;
  level: string;
  reference: string;
}

// Every level in levels/ with its reference solution, bundled as raw JSON.
function fixtures(): Fixture[] {
  const files = import.meta.glob('../../levels/*/{level,reference}.json', {
    query: '?raw',
    import: 'default',
    eager: true,
  }) as Record<string, string>;
  const byId = new Map<string, Partial<Fixture>>();
  for (const [path, text] of Object.entries(files)) {
    const m = /levels\/([^/]+)\/(level|reference)\.json$/.exec(path);
    if (!m) {
      continue;
    }
    const [, id, kind] = m;
    if (id === undefined) {
      continue;
    }
    const entry = byId.get(id) ?? { id };
    if (kind === 'level') {
      entry.level = text;
    } else {
      entry.reference = text;
    }
    byId.set(id, entry);
  }
  const out: Fixture[] = [];
  for (const f of byId.values()) {
    if (f.id !== undefined && f.level !== undefined && f.reference !== undefined) {
      out.push({ id: f.id, level: f.level, reference: f.reference });
    }
  }
  return out.sort((a, b) => a.id.localeCompare(b.id));
}

function rulesVersionOf(levelJson: string): number {
  const parsed: unknown = JSON.parse(levelJson);
  const v = typeof parsed === 'object' && parsed !== null ? (parsed as { rules_version?: unknown }).rules_version : undefined;
  return typeof v === 'number' ? v : -1;
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, text?: string, className?: string): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  if (text !== undefined) {
    e.textContent = text;
  }
  if (className !== undefined) {
    e.className = className;
  }
  return e;
}

async function page(app: HTMLElement): Promise<void> {
  const wasm = await loadWasm();
  const levels = fixtures();
  const select = el('select');
  for (const f of levels) {
    const option = el('option', f.id);
    option.value = f.id;
    select.append(option);
  }
  const playButton = el('button', 'Play');
  const restartButton = el('button', 'Restart');
  const muteButton = el('button', 'Mute');
  const controls = el('div', undefined, 'controls');
  controls.append(select, playButton, restartButton, muteButton);
  const canvas = el('canvas');
  const status = el('p', 'loading', 'status');
  app.replaceChildren(el('h1', 'Clattertrap'), controls, canvas, status);

  const player = new Player(canvas, wasm.dt_s, (text) => {
    status.textContent = text;
    playButton.textContent = player.isPlaying() ? 'Pause' : 'Play';
  });
  const load = (): void => {
    const f = levels.find((x) => x.id === select.value);
    if (!f) {
      status.textContent = 'no levels found';
      return;
    }
    if (rulesVersionOf(f.level) !== wasm.rules_version) {
      status.textContent = `${f.id} was written for rules v${rulesVersionOf(f.level)}; this build runs v${wasm.rules_version}`;
      return;
    }
    try {
      player.load(f.level, f.reference);
    } catch (e) {
      status.textContent = e instanceof Error ? e.message : String(e);
    }
  };
  select.addEventListener('change', load);
  playButton.addEventListener('click', () => player.toggle());
  restartButton.addEventListener('click', () => player.restart());
  let muted = false;
  muteButton.addEventListener('click', () => {
    muted = !muted;
    player.setMuted(muted);
    muteButton.textContent = muted ? 'Unmute' : 'Mute';
  });
  load();
}

const app = document.getElementById('app');
if (app) {
  page(app).catch((e: unknown) => {
    app.textContent = e instanceof Error ? e.message : String(e);
  });
}
