// The voices one step's events ask for, read from the live replay: a contact's volume comes from the
// faster of its two parts at that step, which the event log alone does not carry.

import type { Look, ReplayEvent, Setup } from './decode';
import { type Voice, voiceFor } from './sound';

export interface Sounding {
  setup: Setup;
  speed(part: number): number;
}

export const VOICES_PER_STEP = 6;

function lookOf(setup: Setup, part: number): Look | null {
  const slot = setup.slots.find((s) => s.part === part && s.visual.shape !== 'pivot');
  return slot && slot.visual.shape !== 'pivot' ? slot.visual.look : null;
}

export function voicesAt(replay: Sounding, events: ReplayEvent[], cap: number): Voice[] {
  const voices: Voice[] = [];
  for (const e of events) {
    if (voices.length >= cap) {
      break;
    }
    const speed = e.kind === 'contact' ? Math.max(replay.speed(e.a), replay.speed(e.b)) : 0;
    const v = voiceFor(e, (part) => lookOf(replay.setup, part), speed, replay.setup.goal.kind);
    if (v !== null) {
      voices.push(v);
    }
  }
  return voices;
}
