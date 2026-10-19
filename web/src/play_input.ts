// The editor's pointer and keyboard controls on the play page: dragging on the canvas, the wheel to
// turn, arrows to nudge, brackets to turn, and Delete to remove, each handed to the page.

const NUDGE_CM = 1;
const TURN_DEG = 5;

export interface InputActions {
  // Whether the editor is taking input, as opposed to a run playing.
  editing(): boolean;
  // Whether a part is picked, so turning and nudging have something to act on.
  picked(): boolean;
  down(e: PointerEvent): void;
  move(e: PointerEvent): void;
  up(): void;
  turn(by_deg: number): void;
  nudge(dx_cm: number, dy_cm: number): void;
  remove(): void;
}

const MOVES: Record<string, [number, number]> = {
  ArrowLeft: [-NUDGE_CM, 0],
  ArrowRight: [NUDGE_CM, 0],
  ArrowUp: [0, NUDGE_CM],
  ArrowDown: [0, -NUDGE_CM],
};

function isTyping(e: KeyboardEvent): boolean {
  const t = e.target;
  return t instanceof HTMLInputElement || t instanceof HTMLSelectElement || t instanceof HTMLTextAreaElement;
}

export function installInput(canvas: HTMLCanvasElement, actions: InputActions): void {
  canvas.tabIndex = 0;
  canvas.addEventListener('pointerdown', (e) => {
    if (actions.editing()) {
      canvas.focus();
      actions.down(e);
    }
  });
  canvas.addEventListener('pointermove', (e) => actions.move(e));
  canvas.addEventListener('pointerup', () => actions.up());
  canvas.addEventListener('pointercancel', () => actions.up());
  canvas.addEventListener(
    'wheel',
    (e) => {
      if (actions.editing() && actions.picked()) {
        e.preventDefault();
        actions.turn(e.deltaY > 0 ? -TURN_DEG : TURN_DEG);
      }
    },
    { passive: false },
  );
  document.addEventListener('keydown', (e) => {
    if (!actions.editing() || !actions.picked() || isTyping(e)) {
      return;
    }
    const move = MOVES[e.key];
    if (move) {
      e.preventDefault();
      actions.nudge(move[0], move[1]);
    } else if (e.key === '[' || e.key === ']') {
      actions.turn(e.key === '[' ? TURN_DEG : -TURN_DEG);
    } else if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault();
      actions.remove();
    }
  });
}
