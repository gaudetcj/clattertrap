// A rendered soundtrack as a WAV file: 16-bit PCM with a 44-byte RIFF header, for ffmpeg to mux.

export function encodeWav(channels: Float32Array[], sampleRate: number): Uint8Array {
  const n = channels.length;
  const frames = channels[0]?.length ?? 0;
  const dataBytes = frames * n * 2;
  const out = new Uint8Array(44 + dataBytes);
  const view = new DataView(out.buffer);
  const tag = (at: number, text: string): void => {
    for (let i = 0; i < text.length; i += 1) {
      out[at + i] = text.charCodeAt(i);
    }
  };
  tag(0, 'RIFF');
  view.setUint32(4, 36 + dataBytes, true);
  tag(8, 'WAVE');
  tag(12, 'fmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, n, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * n * 2, true);
  view.setUint16(32, n * 2, true);
  view.setUint16(34, 16, true);
  tag(36, 'data');
  view.setUint32(40, dataBytes, true);
  let at = 44;
  for (let i = 0; i < frames; i += 1) {
    for (const channel of channels) {
      const s = Math.max(-1, Math.min(1, channel[i] ?? 0));
      view.setInt16(at, s < 0 ? Math.round(s * 32768) : Math.round(s * 32767), true);
      at += 2;
    }
  }
  return out;
}

// Base64 in slices, since `btoa` takes a string and a string of megabytes built in one go is slow.
export function toBase64(bytes: Uint8Array): string {
  const parts: string[] = [];
  for (let at = 0; at < bytes.length; at += 32768) {
    parts.push(String.fromCharCode(...bytes.subarray(at, at + 32768)));
  }
  return btoa(parts.join(''));
}
