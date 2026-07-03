import { test, expect } from '@playwright/test';

test('create product and verify list update', async ({ page }) => {
  // Navigate to products page
  await page.goto('/products');

  // Fill out the product creation form
  const uniqueTitle = `Test Product ${Date.now()}`;
  await page.fill('input[name="title"]', uniqueTitle);
  await page.fill('input[name="handle"]', `test-handle-${Date.now()}`);
  await page.fill('input[name="priceCents"]', '1999');
  await page.fill('input[name="inventoryQuantity"]', '50');
  await page.fill('textarea[name="description"]', 'A great test product');
  await page.check('input[name="published"]');

  // Submit form
  await page.click('button[type="submit"]');

  // Wait for success message
  await expect(page.locator('.success-message')).toBeVisible();

  // Verify the product appears in the list (table)
  const tableRow = page.locator('table.modern-table tbody tr', { hasText: uniqueTitle });
  await expect(tableRow).toBeVisible();
});
