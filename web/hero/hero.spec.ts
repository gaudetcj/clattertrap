import { Buffer } from 'node:buffer';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import process from 'node:process';
import { test } from '@playwright/test';
import { gifTakes } from '../src/hero_plan';

// Records the arena grid frame by frame through the page's record hook: a one-second lead-in on
// frame 0, every frame until the grid ends, a two-second hold on the last, then the soundtrack.
// `frames/` holds the whole run for the MP4. `gif/` stops after a window of machine time with the
// still-running lanes stamped by their stored verdicts, so the GIF stays short.
// Reads HERO_LEVEL, HERO_ARM, HERO_OUT, and HERO_WINDOW_S; `cargo xtask hero` sets the first three.
const FPS = 30;
const STEPS_PER_FRAME = 4;
const LEAD_S = 1;
const HOLD_S = 2;

class Frames {
  count = 0;
  constructor(private readonly dir: string) {
    mkdirSync(dir, { recursive: true });
  }
  write(dataUrl: string, copies: number): void {
    const bytes = Buffer.from(dataUrl.slice(dataUrl.indexOf(',') + 1), 'base64');
    for (let i = 0; i < copies; i += 1) {
      this.count += 1;
      writeFileSync(join(this.dir, `${String(this.count).padStart(5, '0')}.png`), bytes);
    }
  }
}

test('record the arena grid', async ({ page }) => {
  const level = process.env['HERO_LEVEL'] ?? 'seesaw-launch';
  const arm = process.env['HERO_ARM'] ?? 'blueprint';
  const out = process.env['HERO_OUT'] ?? 'hero-out';
  const window_s = Number.parseFloat(process.env['HERO_WINDOW_S'] ?? '6');
  const movie = new Frames(join(out, 'frames'));
  const gif = new Frames(join(out, 'gif'));
  await page.goto(`/index.html?level=${encodeURIComponent(level)}&arm=${encodeURIComponent(arm)}&record=1`);
  await page.waitForFunction(() => window.clattertrapRecord !== undefined);
  const first = await page.evaluate(() => window.clattertrapRecord?.frame() ?? '');
  movie.write(first, LEAD_S * FPS);
  gif.write(first, LEAD_S * FPS);
  let ended = false;
  let gifOpen = true;
  while (!ended) {
    const next = await page.evaluate(
      (steps) => {
        const hook = window.clattertrapRecord;
        if (!hook) {
          return { ended: true, frame: '' };
        }
        return { ended: hook.advance(steps), frame: hook.frame() };
      },
      STEPS_PER_FRAME,
    );
    ended = next.ended;
    movie.write(next.frame, ended ? HOLD_S * FPS : 1);
    if (gifOpen && gifTakes(movie.count, LEAD_S, window_s, FPS)) {
      gif.write(next.frame, ended ? HOLD_S * FPS : 1);
    } else if (gifOpen) {
      // The window closed with machines still running: stamp them as the gallery recorded them.
      gif.write(await page.evaluate(() => window.clattertrapRecord?.stampedFrame() ?? ''), HOLD_S * FPS);
      gifOpen = false;
    }
  }
  const total_s = movie.count / FPS;
  const wav = await page.evaluate(([lead, total]) => window.clattertrapRecord?.audio(lead ?? 0, total ?? 0) ?? '', [LEAD_S, total_s]);
  writeFileSync(join(out, 'audio.wav'), Buffer.from(wav, 'base64'));
  writeFileSync(
    join(out, 'meta.json'),
    `${JSON.stringify({ level, arm, fps: FPS, frames: movie.count, seconds: total_s, gif_frames: gif.count, window_s })}\n`,
  );
});
