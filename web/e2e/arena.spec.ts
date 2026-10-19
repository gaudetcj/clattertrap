import { expect, test } from '@playwright/test';

// The six models of the first arena, in panel order; the page's MODEL_ORDER says the same.
const MODELS = [
  'claude-haiku-4-5',
  'claude-sonnet-4-5',
  'claude-opus-4-5',
  'claude-haiku-5-5',
  'claude-sonnet-5-5',
  'claude-opus-5-5',
];

const VERDICT = /^(RANG|PRESSED|FILLED|POPPED) \d+\.\d s$|^MISSED$|^CAME TO REST$/;

test('the arena grid plays six machines and shows their stamps', async ({ page }) => {
  await page.goto('/index.html?level=seesaw-launch&arm=blueprint&autoplay=1');
  const status = page.locator('.status');
  await expect(status).toHaveAttribute('data-ended', 'true', { timeout: 60_000 });
  const stamps = status.locator('.stamp');
  await expect(stamps).toHaveCount(6);
  for (let i = 0; i < MODELS.length; i += 1) {
    await expect(stamps.nth(i)).toHaveAttribute('data-model', MODELS[i] ?? '');
    // A replay that disagrees with its stored result stamps REPLAY MISMATCH, which this refuses.
    await expect(stamps.nth(i)).toHaveText(VERDICT);
  }
  const canvas = page.locator('canvas');
  await expect(canvas).toHaveJSProperty('width', 1200);
  await expect(canvas).toHaveJSProperty('height', 808);
});
