import { expect, test } from '@playwright/test';

// The six Claude models, the lineup the test asks for.
const MODELS = [
  'claude-haiku-4-5',
  'claude-sonnet-4-5',
  'claude-opus-4-5',
  'claude-haiku-5-5',
  'claude-sonnet-5-5',
  'claude-opus-5-5',
];

const VERDICT = /^(RANG|PRESSED|FILLED|POPPED) \d+\.\d s$|^MISSED$|^CAME TO REST$/;

test('the arena grid plays the six Claude machines and shows their stamps', async ({ page }) => {
  await page.goto('/arena.html?level=seesaw-launch&arm=blueprint&lineup=claude&autoplay=1');
  const status = page.locator('.status');
  await expect(status).toHaveAttribute('data-ended', 'true', { timeout: 60_000 });
  const stamps = status.locator('.stamp');
  await expect(stamps).toHaveCount(6);
  // The last stamp lands, holds, and tucks into its header; then the grid has settled.
  await expect(status).toHaveAttribute('data-settled', 'true', { timeout: 10_000 });
  // The lanes follow the leaderboard, so only the set is fixed.
  const shown = await stamps.evaluateAll((spans) => spans.map((s) => s.getAttribute('data-model')));
  expect([...shown].sort()).toEqual([...MODELS].sort());
  for (let i = 0; i < MODELS.length; i += 1) {
    // A replay that disagrees with its stored result stamps REPLAY MISMATCH, which this refuses.
    await expect(stamps.nth(i)).toHaveText(VERDICT);
  }
  const canvas = page.locator('canvas');
  await expect(canvas).toHaveJSProperty('width', 1200);
  await expect(canvas).toHaveJSProperty('height', 808);
});

test('a model with no submission gets a panel stamped from the start', async ({ page }) => {
  await page.goto('/arena.html?level=seesaw-launch&arm=workshop&models=claude-opus-5-5,claude-haiku-5-5');
  const status = page.locator('.status');
  const stamps = status.locator('.stamp');
  await expect(stamps).toHaveCount(2);
  await expect(stamps.nth(1)).toHaveAttribute('data-model', 'claude-haiku-5-5');
  // The miss is stamped before the grid has ended, while the machine beside it still runs.
  await expect(stamps.nth(1)).toHaveText('NO SUBMISSION');
  await expect(status).toHaveAttribute('data-ended', 'true', { timeout: 60_000 });
  await expect(stamps.nth(0)).toHaveText(VERDICT);
});

test('a phone fits the whole grid on one screen, two panels per row', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/arena.html?level=seesaw-launch&arm=blueprint&lineup=claude');
  const canvas = page.locator('canvas');
  await expect(canvas).toHaveJSProperty('width', 800);
  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error('no canvas box');
  }
  expect(box.width).toBeLessThanOrEqual(390);
  expect(box.y + box.height).toBeLessThanOrEqual(844);
  // Two columns fill the width; the status may scroll under the grid.
  expect(box.width).toBeGreaterThan(350);
});

test('a laptop gives the grid the height down to the status line, not the footer', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  await page.goto('/arena.html?level=seesaw-launch&arm=blueprint&lineup=claude');
  const canvas = page.locator('canvas');
  await expect(canvas).toHaveJSProperty('width', 1200);
  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error('no canvas box');
  }
  // The grid takes most of the room under its top edge and still ends above the status line.
  expect(box.height).toBeGreaterThan(0.85 * (768 - box.y));
  const status = await page.locator('.status').boundingBox();
  expect(status?.y).toBeGreaterThanOrEqual(box.y + box.height);
});
