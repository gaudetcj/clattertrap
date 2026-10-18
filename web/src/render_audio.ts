// The arena soundtrack rendered offline: every lane re-simulated from its scene so each contact's
// speed is known, voiced into one OfflineAudioContext at the step's time.

import { Replay } from './replay';
import { Soundtrack } from './sound';
import { VOICES_PER_STEP, voicesAt } from './voices';

export interface AudioLane {
  levelJson: string;
  sceneJson: string;
}

export async function renderAudio(
  lanes: AudioLane[],
  dt_s: number,
  lead_s: number,
  total_s: number,
  sampleRate = 48000,
): Promise<AudioBuffer> {
  const ctx = new OfflineAudioContext(1, Math.max(1, Math.ceil(total_s * sampleRate)), sampleRate);
  const sound = new Soundtrack(ctx);
  sound.setVoices(lanes.length);
  for (const lane of lanes) {
    const replay = Replay.start(lane.levelJson, lane.sceneJson);
    while (replay.outcome() === null) {
      const events = replay.step();
      const at_s = lead_s + replay.stepIndex() * dt_s;
      if (at_s >= total_s) {
        break;
      }
      for (const v of voicesAt(replay, events, VOICES_PER_STEP)) {
        sound.play(v, at_s);
      }
    }
    replay.free();
  }
  return ctx.startRendering();
}
