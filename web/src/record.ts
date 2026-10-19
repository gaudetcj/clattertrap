// The hook a recorder drives: step the grid by hand, read the canvas as PNG, and render the
// soundtrack as WAV. Installed on the arena page with `?record=1`, where nothing is paced.

import type { Conductor } from './conductor';
import { renderAudio, type AudioLane } from './render_audio';
import { encodeWav, toBase64 } from './wav';

export interface RecordHook {
  width: number;
  height: number;
  // Steps every running lane and redraws; true once the grid has ended and every stamp has tucked.
  advance(steps: number): boolean;
  // The whole canvas as a PNG data URL.
  frame(): string;
  // The canvas with the stored verdicts stamped over the lanes still running, as a PNG data URL.
  stampedFrame(): string;
  // The soundtrack from `lead_s` in, `total_s` long, as a base64 WAV.
  audio(lead_s: number, total_s: number): Promise<string>;
}

declare global {
  interface Window {
    clattertrapRecord?: RecordHook;
  }
}

export function installRecorder(canvas: HTMLCanvasElement, conductor: Conductor, lanes: AudioLane[], dt_s: number): void {
  const size = conductor.size();
  window.clattertrapRecord = {
    width: size.w,
    height: size.h,
    advance: (steps) => {
      conductor.advance(steps);
      return conductor.settled();
    },
    frame: () => canvas.toDataURL('image/png'),
    stampedFrame: () => {
      conductor.drawStoredVerdicts();
      return canvas.toDataURL('image/png');
    },
    audio: async (lead_s, total_s) => {
      const buffer = await renderAudio(lanes, dt_s, lead_s, total_s);
      return toBase64(encodeWav([buffer.getChannelData(0)], buffer.sampleRate));
    },
  };
}
