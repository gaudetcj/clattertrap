import { expect, test } from '@playwright/test';

const VERDICT = /^(RANG|PRESSED|FILLED|POPPED) \d+\.\d s$|^MISSED$|^CAME TO REST$/;

test('play mode places a part from the palette, carries it in the link, and runs it', async ({ page }) => {
  await page.goto('/play.html?level=bell-drop');
  const status = page.locator('.status');
  await expect(status).not.toHaveText('loading');
  // The level opens empty, with the palette offering what the inventory holds.
  const parts = page.locator('.parts li');
  await expect(parts).toHaveCount(0);
  await page.locator('.palette button', { hasText: 'beam' }).first().click();
  await expect(parts).toHaveCount(1);
  await expect(parts.first()).toContainText('beam');
  expect(page.url()).toContain('scene=');
  // The selected part's fields are offered with the rules' range.
  const length = page.locator('.fields input[name="length_cm"]');
  await expect(length).toHaveAttribute('min', '20');
  await expect(length).toHaveAttribute('max', '150');
  await page.locator('button', { hasText: 'Run' }).click();
  await expect(status).toHaveAttribute('data-ended', 'true', { timeout: 60_000 });
  await expect(status.locator('.stamp')).toHaveText(VERDICT);
});

test('a shared link opens its scene for editing', async ({ page }) => {
  // {"level":"bell-drop","parts":[{"id":"b1","kind":"beam","x_cm":200,"y_cm":150,"length_cm":100,"angle_deg":0}]}
  const scene = 'eyJsZXZlbCI6ImJlbGwtZHJvcCIsInBhcnRzIjpbeyJpZCI6ImIxIiwia2luZCI6ImJlYW0iLCJ4X2NtIjoyMDAsInlfY20iOjE1MCwibGVuZ3RoX2NtIjoxMDAsImFuZ2xlX2RlZyI6MH1dfQ';
  await page.goto(`/play.html?level=bell-drop&scene=${scene}`);
  const parts = page.locator('.parts li');
  await expect(parts).toHaveCount(1);
  await expect(parts.first()).toContainText('b1');
  await expect(page.locator('select.level')).toHaveValue('bell-drop');
});
