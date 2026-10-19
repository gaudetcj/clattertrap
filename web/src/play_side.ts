// The play page's side column: the parts palette with what is left, the picked part's fields with
// the rules' ranges, and the list of placed parts. It draws from the scene and reports edits back.

import { el } from './dom';
import { fieldsOf, type EditPart, type EditScene, type Ranges, type WorldSize } from './edit_scene';
import { tray } from './tray';

const MATERIALS = ['steel', 'wood', 'rubber'];

// What the page does with the side column's edits.
export interface SideActions {
  place(kind: string): void;
  pick(id: string): void;
  set(id: string, name: string, value: number | string): void;
  remove(id: string): void;
}

export class PlaySide {
  readonly palette = el('div', undefined, 'palette');
  readonly fields = el('div', undefined, 'fields');
  readonly parts = el('ul', undefined, 'parts');

  constructor(
    private readonly ranges: Ranges,
    private readonly actions: SideActions,
  ) {}

  // Brings every box up to date with the scene; `world` is undefined before a level is shown.
  render(levelJson: string, scene: EditScene, picked: string | null, world: WorldSize | undefined): void {
    this.renderPalette(levelJson, scene);
    this.renderFields(scene, picked, world);
    this.renderParts(scene, picked);
  }

  // The fields alone, for an edit that must not rebuild the input being typed in.
  private renderPalette(levelJson: string, scene: EditScene): void {
    this.palette.replaceChildren(
      ...tray(levelJson, JSON.stringify(scene)).map((item) => {
        const button = el('button', `${item.kind} ${item.used}/${item.allowed}`);
        button.disabled = item.used >= item.allowed;
        button.addEventListener('click', () => this.actions.place(item.kind));
        return button;
      }),
    );
  }

  private renderFields(scene: EditScene, picked: string | null, world: WorldSize | undefined): void {
    const part = scene.parts.find((p) => p.id === picked);
    if (!part || !world) {
      this.fields.replaceChildren(el('p', 'Pick a part on the level or in the list.', 'muted'));
      return;
    }
    const rows: HTMLElement[] = [el('span', `${part.id} ${part.kind}`, 'name')];
    const remove = el('button', 'Remove');
    remove.addEventListener('click', () => this.actions.remove(part.id));
    rows.push(remove);
    for (const field of fieldsOf(part.kind, this.ranges, world)) {
      rows.push(el('label', field.name), this.number(part, field.name, field.min, field.max, field.step));
    }
    if (part.kind === 'ball') {
      rows.push(el('label', 'material'), this.choice(part, 'material', MATERIALS));
    }
    if (part.kind === 'rope') {
      const solid = scene.parts.filter((p) => p.kind !== 'rope').map((p) => p.id);
      rows.push(el('label', 'a'), this.choice(part, 'a', solid), el('label', 'b'), this.choice(part, 'b', solid));
    }
    this.fields.replaceChildren(...rows);
  }

  private number(part: EditPart, name: string, min: number, max: number, step: number): HTMLInputElement {
    const input = el('input');
    input.type = 'number';
    input.name = name;
    input.min = String(min);
    input.max = String(max);
    input.step = String(step);
    input.value = String(part[name] ?? 0);
    input.addEventListener('input', () => {
      const value = Number(input.value);
      if (Number.isFinite(value)) {
        this.actions.set(part.id, name, value);
      }
    });
    return input;
  }

  private choice(part: EditPart, name: string, options: string[]): HTMLSelectElement {
    const select = el('select');
    select.name = name;
    for (const text of options) {
      const option = el('option', text);
      option.value = text;
      select.append(option);
    }
    select.value = String(part[name] ?? '');
    select.addEventListener('change', () => this.actions.set(part.id, name, select.value));
    return select;
  }

  private renderParts(scene: EditScene, picked: string | null): void {
    this.parts.replaceChildren(
      ...scene.parts.map((p) => {
        const li = el('li', `${p.id}  ${p.kind}`, p.id === picked ? 'picked' : undefined);
        li.addEventListener('click', () => this.actions.pick(p.id));
        return li;
      }),
    );
  }
}
