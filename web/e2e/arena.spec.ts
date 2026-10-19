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
