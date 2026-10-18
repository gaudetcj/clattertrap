// Sound synthesized from simulator events. `voiceFor` is the pure mapping from an event to a voice;
// `Soundtrack` plays voices on any BaseAudioContext, so an OfflineAudioContext can render the same mix.

import type { GoalKind, Look, ReplayEvent } from './decode';

export type Timbre = 'knock' | 'ping' | 'thud' | 'tick' | 'pop' | 'bell' | 'click' | 'thump';

export interface Voice {
  timbre: Timbre;
  gain: number;
}

const MIN_GAIN = 0.15;
const FULL_SPEED_M_S = 2;

function contactTimbre(a: Look, b: Look): Timbre {
  const pair = [a, b];
  if (pair.includes('brass')) {
    return 'tick';
  }
  if (pair.includes('steel') || pair.includes('pin')) {
    return 'ping';
  }
  if (pair.includes('rubber')) {
    return 'thud';
  }
  return 'knock';
}

export function voiceFor(
  event: ReplayEvent,
  lookOf: (part: number) => Look | null,
  speed_m_s: number,
  goal: GoalKind,
): Voice | null {
  switch (event.kind) {
    case 'contact': {
      const a = lookOf(event.a);
      const b = lookOf(event.b);
      if (a === null || b === null) {
        return null;
      }
      const pair = [a, b];
      if (pair.includes('pin') && pair.includes('balloon')) {
        return { timbre: 'pop', gain: 1 };
      }
      const gain = Math.min(1, Math.max(MIN_GAIN, speed_m_s / FULL_SPEED_M_S));
      return { timbre: contactTimbre(a, b), gain };
    }
    case 'goal_met':
      switch (goal) {
        case 'ring':
          return { timbre: 'bell', gain: 1 };
        case 'press':
          return { timbre: 'click', gain: 1 };
        case 'fill':
          return { timbre: 'thump', gain: 1 };
        case 'pop':
          return null;
      }
      break;
    case 'via':
    case 'at_rest':
    case 'left_world':
      return null;
  }
  return null;
}

// Bell partials relative to the strike tone, see docs/physics.md#bell for what rings it.
const BELL_PARTIALS: [number, number, number][] = [
  [1, 1, 2.2],
  [2.01, 0.5, 1.6],
  [2.98, 0.3, 1.1],
  [4.19, 0.15, 0.7],
];

// The master level for one machine; more machines share it so the mix does not clip.
export const MASTER_GAIN = 0.5;

// The master level for `machines` playing at once.
export function masterFor(machines: number): number {
  return MASTER_GAIN / Math.sqrt(Math.max(1, machines));
}

export class Soundtrack {
  private noise: AudioBuffer | null = null;
  private readonly master: GainNode;

  constructor(private readonly ctx: BaseAudioContext) {
    this.master = ctx.createGain();
    this.master.gain.value = MASTER_GAIN;
    this.master.connect(ctx.destination);
  }

  // Lowers the master for several machines sounding together.
  setVoices(machines: number): void {
    this.master.gain.value = masterFor(machines);
  }

  play(voice: Voice, at_s: number): void {
    switch (voice.timbre) {
      case 'knock':
        this.burst(at_s, voice.gain, 220, 0.05, 'bandpass');
        break;
      case 'thud':
        this.burst(at_s, voice.gain * 0.8, 90, 0.09, 'lowpass');
        break;
      case 'ping':
        this.tone(at_s, voice.gain * 0.5, 1800, 0.12, 'sine');
        this.burst(at_s, voice.gain * 0.3, 2500, 0.02, 'highpass');
        break;
      case 'tick':
        this.tone(at_s, voice.gain * 0.3, 1400, 0.05, 'triangle');
        break;
      case 'click':
        this.burst(at_s, voice.gain * 0.6, 3000, 0.02, 'highpass');
        this.tone(at_s + 0.02, voice.gain * 0.4, 600, 0.05, 'square');
        break;
      case 'thump':
        this.burst(at_s, voice.gain, 120, 0.15, 'lowpass');
        break;
      case 'pop':
        this.burst(at_s, voice.gain, 1200, 0.08, 'bandpass');
        this.burst(at_s, voice.gain * 0.7, 300, 0.12, 'lowpass');
        break;
      case 'bell':
        for (const [ratio, level, decay_s] of BELL_PARTIALS) {
          this.tone(at_s, voice.gain * level * 0.5, 880 * ratio, decay_s, 'sine');
        }
        break;
    }
  }

  private tone(at_s: number, gain: number, hz: number, decay_s: number, type: OscillatorType): void {
    const osc = this.ctx.createOscillator();
    osc.type = type;
    osc.frequency.value = hz;
    const env = this.envelope(at_s, gain, decay_s);
    osc.connect(env);
    osc.start(at_s);
    osc.stop(at_s + decay_s + 0.05);
  }

  private burst(at_s: number, gain: number, hz: number, decay_s: number, type: BiquadFilterType): void {
    const src = this.ctx.createBufferSource();
    src.buffer = this.noiseBuffer();
    const filter = this.ctx.createBiquadFilter();
    filter.type = type;
    filter.frequency.value = hz;
    filter.Q.value = 1.2;
    const env = this.envelope(at_s, gain, decay_s);
    src.connect(filter);
    filter.connect(env);
    src.start(at_s);
    src.stop(at_s + decay_s + 0.05);
  }

  // A fast attack and an exponential decay into the master gain.
  private envelope(at_s: number, gain: number, decay_s: number): GainNode {
    const g = this.ctx.createGain();
    g.gain.setValueAtTime(0.0001, at_s);
    g.gain.exponentialRampToValueAtTime(Math.max(0.0001, gain), at_s + 0.004);
    g.gain.exponentialRampToValueAtTime(0.0001, at_s + decay_s);
    g.connect(this.master);
    return g;
  }

  private noiseBuffer(): AudioBuffer {
    if (this.noise === null) {
      const rate = this.ctx.sampleRate;
      const buffer = this.ctx.createBuffer(1, rate, rate);
      const data = buffer.getChannelData(0);
      // A fixed linear congruential sequence so the same events give the same waveform every time.
      let seed = 2463534242;
      for (let i = 0; i < data.length; i += 1) {
        seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
        data[i] = seed / 2147483648 - 1;
      }
      this.noise = buffer;
    }
    return this.noise;
  }
}
