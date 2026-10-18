// The one DOM helper the pages share.

export function el<K extends keyof HTMLElementTagNameMap>(tag: K, text?: string, className?: string): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  if (text !== undefined) {
    e.textContent = text;
  }
  if (className !== undefined) {
    e.className = className;
  }
  return e;
}
