// The room a page gives its grid canvas: the container's inner width, and the height left on screen
// after everything under the canvas. A phone takes the height down from the canvas to the bottom of
// the small viewport, the one with the address bar shown, so the whole grid fits one screen and
// never changes size as the bar comes and goes; the lines under the grid scroll beneath it.

// The screens page.css lays out in one column.
const NARROW = '(max-width: 720px)';

function px(value: string): number {
  const n = Number.parseFloat(value);
  return Number.isFinite(n) ? n : 0;
}

// The height the lines under `canvas` take: its later siblings up to the page footer, with their
// margins and the gaps between them, and a little room. The footer may scroll off the screen.
function heightBelow(canvas: HTMLElement): number {
  const parent = canvas.parentElement;
  if (!parent) {
    return 0;
  }
  const style = getComputedStyle(parent);
  const gap = px(style.rowGap);
  let total = px(getComputedStyle(canvas).marginBottom) + 12;
  let after = false;
  for (const child of Array.from(parent.children)) {
    if (child === canvas) {
      after = true;
    } else if (after && child.tagName === 'FOOTER') {
      break;
    } else if (after) {
      const cs = getComputedStyle(child);
      total += child.getBoundingClientRect().height + px(cs.marginTop) + px(cs.marginBottom) + gap;
    }
  }
  return total;
}

// The width the canvas may take: its container's inner width, without the padding.
function widthWithin(canvas: HTMLElement): number {
  const parent = canvas.parentElement;
  if (!parent) {
    return canvas.getBoundingClientRect().width;
  }
  const style = getComputedStyle(parent);
  return Math.max(1, parent.clientWidth - px(style.paddingLeft) - px(style.paddingRight));
}

// The height of the small viewport, read from a fixed probe of 100svh, which stays put while the
// address bar shows and hides; the window's inner height is the fallback where svh is unknown.
function smallViewportHeight(): number {
  if (!CSS.supports('height', '100svh')) {
    return window.innerHeight;
  }
  const probe = document.createElement('div');
  probe.style.cssText = 'position: fixed; top: 0; left: 0; width: 0; height: 100svh; visibility: hidden; pointer-events: none;';
  document.body.append(probe);
  const h = probe.clientHeight;
  probe.remove();
  return h > 0 ? h : window.innerHeight;
}

export function roomFor(canvas: HTMLElement): { w: number; h: number } {
  const top = canvas.getBoundingClientRect().top + window.scrollY;
  const h = window.matchMedia(NARROW).matches ? smallViewportHeight() - top - 12 : window.innerHeight - top - heightBelow(canvas);
  return { w: widthWithin(canvas), h: Math.max(1, h) };
}
