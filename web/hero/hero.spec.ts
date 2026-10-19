import { Buffer } from 'node:buffer';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import process from 'node:process';
import { test } from '@playwright/test';
import { gifTakes } from '../src/hero_plan';

// Records the arena page frame by frame: the page's record hook steps the grid, and each frame is
// a screenshot of the page's main region, chrome and all, with animations frozen. A one-second
// lead-in on frame 0, every frame until the grid ends, a two-second hold on the last, then the
// soundtrack. `frames/` holds the whole run for the MP4. `gif/` stops after a window of machine
// time with the still-running lanes stamped by their stored verdicts, so the GIF stays short.
// Reads HERO_LEVEL, HERO_ARM, HERO_LINEUP, HERO_OUT, and HERO_WINDOW_S; `cargo xtask hero` sets the first four.
const FPS = 30;
const STEPS_PER_FRAME = 4;
const LEAD_S = 1;
const HOLD_S = 2;
// Tall enough for the six-panel grid at one pixel per centimetre under the title band.
const VIEWPORT = { width: 1280, height: 1080 };

class Frames {
  count = 0;
  constructor(private readonly dir: string) {
    mkdirSync(dir, { recursive: true });
  }
  write(bytes: Buffer, copies: number): void {
    for (let i = 0; i < copies; i += 1) {
      this.count += 1;
      writeFileSync(join(this.dir, `${String(this.count).padStart(5, '0')}.png`), bytes);
    }
  }
}

test('record the arena page', async ({ page }) => {
  const level = process.env['HERO_LEVEL'] ?? 'seesaw-launch';
  const arm = process.env['HERO_ARM'] ?? 'blueprint';
  const lineup = process.env['HERO_LINEUP'] ?? 'families';
  const out = process.env['HERO_OUT'] ?? 'hero-out';
  const window_s = Number.parseFloat(process.env['HERO_WINDOW_S'] ?? '6');
  const movie = new Frames(join(out, 'frames'));
  const gif = new Frames(join(out, 'gif'));
  await page.setViewportSize(VIEWPORT);
  // Reduced motion keeps the marquee readable as still text instead of parked off-screen.
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto(`/arena.html?level=${encodeURIComponent(level)}&arm=${encodeURIComponent(arm)}&lineup=${encodeURIComponent(lineup)}&record=1`);
  await page.waitForFunction(() => window.clattertrapRecord !== undefined);
  await page.evaluate(() => document.fonts.ready);
  const shot = (): Promise<Buffer> => page.locator('main').screenshot({ animations: 'disabled' });
  const first = await shot();
  movie.write(first, LEAD_S * FPS);
  gif.write(first, LEAD_S * FPS);
  let ended = false;
  let gifOpen = true;
  while (!ended) {
    ended = await page.evaluate((steps) => window.clattertrapRecord?.advance(steps) ?? true, STEPS_PER_FRAME);
    const frame = await shot();
    movie.write(frame, ended ? HOLD_S * FPS : 1);
    if (gifOpen && gifTakes(movie.count, LEAD_S, window_s, FPS)) {
      gif.write(frame, ended ? HOLD_S * FPS : 1);
    } else if (gifOpen) {
      // The window closed with machines still running: stamp them as the gallery recorded them.
      await page.evaluate(() => {
        window.clattertrapRecord?.stampedFrame();
      });
      gif.write(await shot(), HOLD_S * FPS);
      gifOpen = false;
    }
  }
  const total_s = movie.count / FPS;
  const wav = await page.evaluate(([lead, total]) => window.clattertrapRecord?.audio(lead ?? 0, total ?? 0) ?? '', [LEAD_S, total_s]);
  writeFileSync(join(out, 'audio.wav'), Buffer.from(wav, 'base64'));
  writeFileSync(
    join(out, 'meta.json'),
    `${JSON.stringify({ level, arm, lineup, fps: FPS, frames: movie.count, seconds: total_s, gif_frames: gif.count, window_s })}\n`,
  );
});
