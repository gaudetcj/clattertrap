import { describe, expect, it } from 'vitest';
import { encodeWav, toBase64 } from './wav';

function ascii(bytes: Uint8Array, at: number, n: number): string {
  return String.fromCharCode(...bytes.slice(at, at + n));
}

function u32(bytes: Uint8Array, at: number): number {
  return new DataView(bytes.buffer, bytes.byteOffset).getUint32(at, true);
}

function u16(bytes: Uint8Array, at: number): number {
  return new DataView(bytes.buffer, bytes.byteOffset).getUint16(at, true);
}

function i16(bytes: Uint8Array, at: number): number {
  return new DataView(bytes.buffer, bytes.byteOffset).getInt16(at, true);
}

describe('encodeWav', () => {
  it('writes a 44-byte RIFF header for 16-bit PCM', () => {
    const wav = encodeWav([new Float32Array([0, 0.5])], 48000);
    expect(wav.length).toBe(44 + 4);
    expect(ascii(wav, 0, 4)).toBe('RIFF');
    expect(u32(wav, 4)).toBe(36 + 4);
    expect(ascii(wav, 8, 4)).toBe('WAVE');
    expect(ascii(wav, 12, 4)).toBe('fmt ');
    expect(u32(wav, 16)).toBe(16);
    expect(u16(wav, 20)).toBe(1);
    expect(u16(wav, 22)).toBe(1);
    expect(u32(wav, 24)).toBe(48000);
    expect(u32(wav, 28)).toBe(96000);
    expect(u16(wav, 32)).toBe(2);
    expect(u16(wav, 34)).toBe(16);
    expect(ascii(wav, 36, 4)).toBe('data');
    expect(u32(wav, 40)).toBe(4);
  });

  it('scales and clamps samples to 16 bits', () => {
    const wav = encodeWav([new Float32Array([1, -1.5, 0, 0.5])], 8000);
    expect([i16(wav, 44), i16(wav, 46), i16(wav, 48), i16(wav, 50)]).toEqual([32767, -32768, 0, 16384]);
  });

  it('interleaves channels', () => {
    const wav = encodeWav([new Float32Array([0.25, 0.5]), new Float32Array([-0.25, -0.5])], 8000);
    expect(u16(wav, 22)).toBe(2);
    expect(u16(wav, 32)).toBe(4);
    expect([i16(wav, 44), i16(wav, 46), i16(wav, 48), i16(wav, 50)]).toEqual([8192, -8192, 16384, -16384]);
  });
});

describe('toBase64', () => {
  it('round-trips bytes of any length', () => {
    const bytes = new Uint8Array(70000);
    for (let i = 0; i < bytes.length; i += 1) {
      bytes[i] = (i * 7) & 0xff;
    }
    const back = Uint8Array.from(atob(toBase64(bytes)), (c) => c.charCodeAt(0));
    expect(back).toEqual(bytes);
    expect(toBase64(new Uint8Array([]))).toBe('');
  });
});
