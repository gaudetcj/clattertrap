import { Buffer } from 'node:buffer';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import process from 'node:process';
import { test } from '@playwright/test';

// Records the arena grid frame by frame through the page's record hook: a one-second lead-in on
// frame 0, every frame until the grid ends, a two-second hold on the last, then the soundtrack.
// Reads HERO_LEVEL, HERO_ARM, and HERO_OUT; `cargo xtask hero` sets them.
const FPS = 30;
const STEPS_PER_FRAME = 4;
const LEAD_S = 1;
const HOLD_S = 2;

test('record the arena grid', async ({ page }) => {
  const level = process.env['HERO_LEVEL'] ?? 'seesaw-launch';
  const arm = process.env['HERO_ARM'] ?? 'blueprint';
  const out = process.env['HERO_OUT'] ?? 'hero-out';
  const frames = join(out, 'frames');
  mkdirSync(frames, { recursive: true });
  await page.goto(`/arena.html?level=${encodeURIComponent(level)}&arm=${encodeURIComponent(arm)}&record=1`);
  await page.waitForFunction(() => window.clattertrapRecord !== undefined);
  let count = 0;
  const write = (dataUrl: string, copies: number): void => {
    const bytes = Buffer.from(dataUrl.slice(dataUrl.indexOf(',') + 1), 'base64');
    for (let i = 0; i < copies; i += 1) {
      count += 1;
      writeFileSync(join(frames, `${String(count).padStart(5, '0')}.png`), bytes);
    }
  };
  write(await page.evaluate(() => window.clattertrapRecord?.frame() ?? ''), LEAD_S * FPS);
  let ended = false;
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
    write(next.frame, ended ? HOLD_S * FPS : 1);
  }
  const total_s = count / FPS;
  const wav = await page.evaluate(([lead, total]) => window.clattertrapRecord?.audio(lead ?? 0, total ?? 0) ?? '', [LEAD_S, total_s]);
  writeFileSync(join(out, 'audio.wav'), Buffer.from(wav, 'base64'));
  writeFileSync(join(out, 'meta.json'), `${JSON.stringify({ level, arm, fps: FPS, frames: count, seconds: total_s })}\n`);
});
