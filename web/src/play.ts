// The play page: build a machine on a level by hand, run it under the same rules the models get,
// and share it as a link. Page state lives in one object; the scene itself is edited by pure functions.

import type { PanelChrome } from './arena_view';
import { band, footer, navLinks, siteCounts } from './chrome';
import { Conductor } from './conductor';
import { el } from './dom';
import { addPart, defaultPart, moveTo, nextId, removePart, sceneJson, setField, type EditScene, type Ranges } from './edit_scene';
import { EditorView } from './editor_view';
import { galleryEntries, type Entry } from './gallery';
import { fixtures, goalKindOf, rulesVersionOf, titleOf, type Fixture } from './levels';
import { goalLine, modelsLine } from './models_here';
import { installInput } from './play_input';
import { PlaySide } from './play_side';
import { param, search } from './query';
import { loadWasm, ranges, Replay } from './replay';
import { decodeScene, encodeScene } from './share';

const DEFAULT_LEVEL = 'bell-drop';

class PlayPage {
  private scene: EditScene = { level: DEFAULT_LEVEL, parts: [] };
  private picked: string | null = null;
  private problems: string[] = [];
  private mode: 'edit' | 'run' = 'edit';
  private drag: { id: string; dx_cm: number; dy_cm: number } | null = null;
  private readonly canvas = el('canvas');
  private readonly status = el('p', 'loading', 'status');
  private readonly problemList = el('ul', undefined, 'problems');
  private readonly models = el('p', undefined, 'models');
  private readonly levelSelect = el('select', undefined, 'level');
  private readonly runButton = el('button', 'Run');
  private readonly arenaLink = el('a', 'watch the models');
  private readonly side: PlaySide;
  private readonly editor: EditorView;
  private readonly conductor: Conductor;

  constructor(
    app: HTMLElement,
    dt_s: number,
    private readonly rules_version: number,
    private readonly levels: Fixture[],
    private readonly entries: Entry[],
    private readonly ranges: Ranges,
  ) {
    this.side = new PlaySide(ranges, {
      place: (kind) => this.place(kind),
      pick: (id) => {
        this.picked = id;
        this.refresh();
      },
      set: (id, name, value) => {
        this.scene = setField(this.scene, id, name, value);
        this.repaint();
      },
      remove: (id) => {
        this.scene = removePart(this.scene, id);
        this.picked = null;
        this.refresh();
      },
    });
    for (const f of levels) {
      const option = el('option', f.id);
      option.value = f.id;
      this.levelSelect.append(option);
    }
    const label = el('label', 'puzzle ', 'pick');
    label.append(this.levelSelect);
    const backButton = el('button', 'Back to editing');
    const clearButton = el('button', 'Clear');
    const linkButton = el('button', 'Copy link');
    this.arenaLink.className = 'open';
    const controls = el('div', undefined, 'controls');
    controls.append(label, this.runButton, backButton, clearButton, linkButton, this.arenaLink);
    const hint = el(
      'p',
      'Pick a part, then drag it into place. Scroll over it or press [ and ] to turn it; arrows nudge it; Delete removes it. Run when nothing is left to fix.',
      'hint',
    );
    const stage = el('div', undefined, 'stage');
    stage.append(this.canvas, this.status, this.problemList);
    const aside = el('aside', undefined, 'side');
    aside.append(this.box('parts', this.side.palette), this.box('picked part', this.side.fields), this.box('placed', this.side.parts), this.box('the models', this.models));
    const play = el('div', undefined, 'play');
    play.append(stage, aside);
    const nav = navLinks([
      ['home', 'index.html'],
      ['arena', 'arena.html'],
      ['gallery', 'gallery.html'],
      ['leaderboard', 'leaderboard.html'],
      ['level', 'level.html'],
    ]);
    const counts = siteCounts(entries, levels);
    app.replaceChildren(band('Clattertrap play', nav, counts), controls, hint, play, footer(counts));
    this.editor = new EditorView(this.canvas);
    this.conductor = new Conductor(this.canvas, dt_s, true, (c) => this.changed(c));
    this.levelSelect.addEventListener('change', () => this.open(this.levelSelect.value, null));
    this.runButton.addEventListener('click', () => this.run());
    backButton.addEventListener('click', () => this.edit());
    clearButton.addEventListener('click', () => this.open(this.scene.level, null));
    linkButton.addEventListener('click', () => {
      void navigator.clipboard.writeText(location.href).then(() => {
        this.status.textContent = 'link copied';
      });
    });
    installInput(this.canvas, {
      editing: () => this.mode === 'edit',
      picked: () => this.picked !== null,
      down: (e) => this.pointerDown(e),
      move: (e) => this.pointerMove(e),
      up: () => this.pointerUp(),
      turn: (by) => this.turn(by),
      nudge: (dx, dy) => this.nudge(dx, dy),
      remove: () => this.removePicked(),
    });
    window.addEventListener('resize', () => (this.mode === 'edit' ? this.editor.fit() : this.conductor.fit()));
  }

  private box(title: string, body: HTMLElement): HTMLElement {
    const box = el('div', undefined, 'box');
    box.append(el('h3', title), body);
    return box;
  }

  private level(): Fixture | undefined {
    return this.levels.find((f) => f.id === this.scene.level);
  }

  private chrome(replay: Replay): PanelChrome {
    const f = this.level();
    return { label: 'YOU', name: f ? titleOf(f.level) : this.scene.level, prediction: goalLine(replay.setup.goal) };
  }

  // Opens a level with a scene, or empty, for editing.
  open(levelId: string, scene: EditScene | null): void {
    this.scene = scene ?? { level: levelId, parts: [] };
    this.picked = null;
    this.levelSelect.value = levelId;
    this.arenaLink.href = `arena.html${search({ level: levelId })}`;
    const f = this.level();
    this.models.textContent = f ? modelsLine(this.entries, levelId, goalKindOf(f.level) ?? 'ring') : '';
    this.edit();
  }

  // Back to the editor, showing the scene as it is.
  private edit(): void {
    this.mode = 'edit';
    this.conductor.unload();
    this.status.textContent = 'editing';
    delete this.status.dataset['ended'];
    this.refresh();
  }

  private address(): void {
    const params: Record<string, string> = { level: this.scene.level };
    if (this.scene.parts.length > 0) {
      params['scene'] = encodeScene(this.scene);
    }
    history.replaceState(null, '', search(params));
  }

  // Builds the scene for drawing, paints it, and brings every box up to date.
  private refresh(): void {
    this.repaint();
    const f = this.level();
    const setup = this.editor.shown()?.setup;
    this.side.render(f?.level ?? '', this.scene, this.picked, setup ? { width_cm: setup.width_cm, height_cm: setup.height_cm } : undefined);
  }

  // Paints and lists the problems, leaving the side column as it is.
  private repaint(): void {
    this.address();
    this.paint();
    this.renderProblems();
  }

  private paint(): void {
    const f = this.level();
    if (!f) {
      this.problems = [`no level ${this.scene.level}`];
      return;
    }
    if (rulesVersionOf(f.level) !== this.rules_version) {
      this.problems = [`${f.id} was written for rules v${rulesVersionOf(f.level)}; this build runs v${this.rules_version}`];
      return;
    }
    const json = sceneJson(this.scene);
    try {
      const preview = Replay.preview(f.level, json);
      this.problems = preview.problems;
      if (this.mode === 'edit') {
        this.editor.show(preview.replay, f.level, json, this.chrome(preview.replay), this.picked);
      } else {
        preview.replay.free();
      }
    } catch (e) {
      this.problems = (e instanceof Error ? e.message : String(e)).split('\n');
    }
  }

  // Adds a part of a kind in the middle of the first build zone and picks it up.
  private place(kind: string): void {
    const shown = this.editor.shown();
    const zone = shown?.setup.zones[0];
    const x_cm = zone ? zone.x_cm + zone.width_cm / 2 : (shown?.setup.width_cm ?? 400) / 2;
    const y_cm = zone ? zone.y_cm + zone.height_cm / 2 : (shown?.setup.height_cm ?? 300) / 2;
    const id = nextId(this.scene, kind);
    let part = defaultPart(kind, id, Math.round(x_cm), Math.round(y_cm), this.ranges);
    if (kind === 'rope') {
      const solid = this.scene.parts.filter((p) => p.kind !== 'rope').map((p) => p.id);
      if (solid.length < 2) {
        this.status.textContent = 'a rope ties two placed parts; place them first';
        return;
      }
      part = { ...part, a: solid[0] ?? '', b: solid[1] ?? '' };
    }
    this.scene = addPart(this.scene, part);
    this.picked = id;
    this.refresh();
  }

  private renderProblems(): void {
    this.problemList.replaceChildren(...this.problems.map((p) => el('li', p)));
    this.runButton.disabled = this.problems.length > 0 || this.mode === 'run';
  }

  private pointerDown(e: PointerEvent): void {
    const ids = new Set(this.scene.parts.map((p) => p.id));
    const name = this.editor.partUnder(e, ids);
    const at = this.editor.worldAt(e);
    this.picked = name ?? null;
    const part = this.scene.parts.find((p) => p.id === name);
    if (name !== undefined && part && at && typeof part['x_cm'] === 'number' && typeof part['y_cm'] === 'number') {
      this.drag = { id: name, dx_cm: part['x_cm'] - at.x_m * 100, dy_cm: part['y_cm'] - at.y_m * 100 };
      this.canvas.setPointerCapture(e.pointerId);
    }
    this.refresh();
  }

  private pointerMove(e: PointerEvent): void {
    const at = this.editor.worldAt(e);
    if (!this.drag || !at) {
      return;
    }
    this.scene = moveTo(this.scene, this.drag.id, at.x_m * 100 + this.drag.dx_cm, at.y_m * 100 + this.drag.dy_cm);
    this.paint();
    this.renderProblems();
  }

  private pointerUp(): void {
    if (this.drag) {
      this.drag = null;
      this.refresh();
    }
  }

  private turn(by: number): void {
    const part = this.scene.parts.find((p) => p.id === this.picked);
    if (!part || typeof part['angle_deg'] !== 'number') {
      return;
    }
    this.scene = setField(this.scene, part.id, 'angle_deg', part['angle_deg'] + by);
    this.refresh();
  }

  private nudge(dx_cm: number, dy_cm: number): void {
    const part = this.scene.parts.find((p) => p.id === this.picked);
    if (!part || typeof part['x_cm'] !== 'number' || typeof part['y_cm'] !== 'number') {
      return;
    }
    this.scene = moveTo(this.scene, part.id, part['x_cm'] + dx_cm, part['y_cm'] + dy_cm);
    this.refresh();
  }

  private removePicked(): void {
    if (this.picked !== null) {
      this.scene = removePart(this.scene, this.picked);
      this.picked = null;
      this.refresh();
    }
  }

  // Runs the scene as a submission would run: the strict rules, the clock, and the stamp.
  private run(): void {
    const f = this.level();
    if (!f || this.problems.length > 0) {
      return;
    }
    const json = sceneJson(this.scene);
    try {
      const preview = Replay.preview(f.level, json);
      const chrome = this.chrome(preview.replay);
      preview.replay.free();
      this.conductor.load([{ chrome, levelJson: f.level, sceneJson: json, stored: null }]);
    } catch (e) {
      this.problems = (e instanceof Error ? e.message : String(e)).split('\n');
      this.renderProblems();
      return;
    }
    this.mode = 'run';
    this.renderProblems();
    this.conductor.play();
  }

  private changed(c: Conductor): void {
    if (this.mode !== 'run') {
      return;
    }
    this.status.dataset['ended'] = String(c.ended());
    const stamp = c.verdicts()[0] ?? null;
    this.status.replaceChildren(el('span', `${c.state()} `), ...(stamp ? [el('span', stamp.text, 'stamp')] : []));
  }
}

async function page(app: HTMLElement): Promise<void> {
  const query = location.search;
  const wasm = await loadWasm();
  const levels = fixtures();
  const play = new PlayPage(app, wasm.dt_s, wasm.rules_version, levels, galleryEntries(), ranges());
  const shared = param(query, 'scene');
  const scene = shared === undefined ? undefined : decodeScene(shared);
  const level = scene?.level ?? param(query, 'level') ?? DEFAULT_LEVEL;
  play.open(levels.some((f) => f.id === level) ? level : DEFAULT_LEVEL, scene ?? null);
}

const app = document.getElementById('app');
if (app) {
  page(app).catch((e: unknown) => {
    app.textContent = e instanceof Error ? e.message : String(e);
  });
}
