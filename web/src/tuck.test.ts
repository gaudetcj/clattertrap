import { describe, expect, it } from 'vitest';
import { HOLD_S, stampPlace, tuckedBox } from './tuck';

describe('where a stamp sits', () => {
  it('lands over the machine, then tucks into the header once the hold has passed', () => {
    expect(HOLD_S).toBe(1.2);
    expect(stampPlace(0)).toBe('landed');
    expect(stampPlace(1.19)).toBe('landed');
    expect(stampPlace(1.2)).toBe('tucked');
    expect(stampPlace(30)).toBe('tucked');
  });

  it('tucks at the label size on the first header line, right-aligned, when it fits beside the label', () => {
    const header = { x: 400, y: 404, w: 400, h: 40 };
    // Ten characters at 2 px are 118 wide; the border and the side pad add 2 px each per side.
    expect(tuckedBox(header, 'OPUS 5.5', 'RANG 4.2 s')).toEqual({ x: 670, y: 406, w: 126, h: 18, px: 2 });
  });

  it('drops to the font size when the label leaves no room', () => {
    const header = { x: 400, y: 404, w: 400, h: 40 };
    // 4 + 226 + 8 + 174 + 4 is 416, more than the 400 the header has.
    expect(tuckedBox(header, 'DEEPSEEK V4.1 FLASH', 'PRESSED 12.5 s')).toEqual({ x: 709, y: 406, w: 87, h: 9, px: 1 });
  });
});
