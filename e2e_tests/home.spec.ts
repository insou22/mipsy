import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.goto('/', { waitUntil: 'load' });
});

test('has title', async ({ page }) => {
  // Expect a title "to contain" a substring.
  await expect(page).toHaveTitle(/mipsy/);
});

test('ui meets snapshot', async ({ page }) => {
  await expect(page).toHaveScreenshot({ maxDiffPixels: 200 });
});
