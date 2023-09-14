import { test, expect } from '@playwright/test';

test('matches snapshot', async ({ page }) => {
  await page.goto('http://localhost:8080/');

  expect(await page.screenshot()).toMatchSnapshot('home.png');

});

test('get started link', async ({ page }) => {
  await page.goto('/');

  // Click the get started link.
  await page.getByRole('link', { name: 'Get started' }).click();

  // Expects page to have a heading with the name of Installation.
  await expect(page.getByRole('heading', { name: 'Installation' })).toBeVisible();
});
