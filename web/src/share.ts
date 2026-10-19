// A scene carried in a link: compact JSON as URL-safe base64 without padding, so a solution can be
// shared as an address and opened back into the editor.

import { parseScene, sceneJson, type EditScene } from './edit_scene';

function toBase64Url(bytes: Uint8Array): string {
  let binary = '';
  for (const b of bytes) {
    binary += String.fromCharCode(b);
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function fromBase64Url(text: string): Uint8Array | undefined {
  if (!/^[A-Za-z0-9_-]+$/.test(text)) {
    return undefined;
  }
  const padded = text.replace(/-/g, '+').replace(/_/g, '/') + '='.repeat((4 - (text.length % 4)) % 4);
  try {
    return Uint8Array.from(atob(padded), (c) => c.charCodeAt(0));
  } catch {
    return undefined;
  }
}

export function encodeScene(scene: EditScene): string {
  return toBase64Url(new TextEncoder().encode(sceneJson(scene)));
}

// The scene a link carries, or undefined when the text is not one.
export function decodeScene(text: string): EditScene | undefined {
  const bytes = fromBase64Url(text);
  if (bytes === undefined) {
    return undefined;
  }
  try {
    return parseScene(new TextDecoder().decode(bytes));
  } catch {
    return undefined;
  }
}
