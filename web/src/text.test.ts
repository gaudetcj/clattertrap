import { describe, expect, it } from 'vitest';
import { FALLBACK, legible, wrapText } from './text';

describe('legible', () => {
  it('keeps letters, digits, and the punctuation predictions use', () => {
    const text = "Ball rolls down the 45-degree ramp (100% sure), hits 'dom1'; rings? Yes! x=2+3";
    expect(legible(text)).toBe(text);
  });

  it('replaces characters without a glyph by the hollow box and folds whitespace', () => {
    expect(legible('rises ≈ 30°\nthen\tstops')).toBe(`rises ${FALLBACK} 30° then stops`);
    expect(legible('')).toBe('');
  });
});

describe('wrapText', () => {
  it('wraps on spaces without exceeding the width', () => {
    expect(wrapText('the quick brown fox jumps over the lazy dog', 10, 9)).toEqual([
      'the quick',
      'brown fox',
      'jumps over',
      'the lazy',
      'dog',
    ]);
  });

  it('splits a token longer than the width', () => {
    expect(wrapText('abcdefghijkl mn', 5, 9)).toEqual(['abcde', 'fghij', 'kl mn']);
  });

  it('fits a 280-character sentence in five lines of 66', () => {
    const sentence =
      'The steel ball drops onto the raised left arm of the seesaw, flips it counterclockwise so the ' +
      'right arm swings up under the box, knocks the wood ball off the shelf onto the spring pad, ' +
      'which launches it over the post and into the bell, ringing it in about four seconds flat.';
    expect(sentence.length).toBeLessThanOrEqual(280);
    const lines = wrapText(sentence, 66, 5);
    expect(lines.length).toBeLessThanOrEqual(5);
    for (const line of lines) {
      expect(line.length).toBeLessThanOrEqual(66);
    }
    expect(lines.join(' ')).toBe(sentence);
  });

  it('clamps to the line count and marks the cut', () => {
    const lines = wrapText('one two three four five six', 5, 2);
    expect(lines).toEqual(['one', 'tw...']);
  });

  it('gives nothing for empty text', () => {
    expect(wrapText('', 10, 3)).toEqual([]);
    expect(wrapText('   ', 10, 3)).toEqual([]);
  });
});
