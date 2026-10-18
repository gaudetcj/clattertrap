// Text for the pixel font: what it can draw, and how a sentence breaks into lines of a panel.

import { hasGlyph } from './font';

// Drawn in place of any character the font lacks.
export const FALLBACK = '□';

// The text with every whitespace run folded to one space and every unknown character boxed.
export function legible(text: string): string {
  return Array.from(text.replace(/\s+/g, ' '), (ch) => (hasGlyph(ch) ? ch : FALLBACK)).join('');
}

// Greedy word wrap to `maxChars` per line; a longer word is cut. Past `maxLines` the text stops
// with `...` on the last line.
export function wrapText(text: string, maxChars: number, maxLines: number): string[] {
  const lines: string[] = [];
  let line = '';
  const push = (word: string): void => {
    if (line.length === 0) {
      line = word;
    } else if (line.length + 1 + word.length <= maxChars) {
      line = `${line} ${word}`;
    } else {
      lines.push(line);
      line = word;
    }
  };
  for (const word of text.split(' ').filter((w) => w.length > 0)) {
    for (let at = 0; at < word.length; at += maxChars) {
      push(word.slice(at, at + maxChars));
    }
  }
  if (line.length > 0) {
    lines.push(line);
  }
  if (lines.length <= maxLines) {
    return lines;
  }
  const kept = lines.slice(0, maxLines);
  const last = kept[maxLines - 1] ?? '';
  kept[maxLines - 1] = `${last.slice(0, Math.max(0, Math.min(last.length, maxChars - 3)))}...`;
  return kept;
}
