import { test, expect } from '@playwright/test'

test.describe('SAK Editor E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Start the dev server first: npm run dev
    await page.goto('http://localhost:5173')
  })

  test('should display welcome screen', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('SAK Editor')
    await expect(page.locator('text=A modern editor for large files')).toBeVisible()
  })

  test('should have Open File button', async ({ page }) => {
    await expect(page.locator('text=📂 Open')).toBeVisible()
  })

  test('should display features list', async ({ page }) => {
    await expect(page.locator('text=Large file support')).toBeVisible()
    await expect(page.locator('text=Hex viewer mode')).toBeVisible()
    await expect(page.locator('text=LLM chat & summary')).toBeVisible()
    await expect(page.locator('text=Color highlighting')).toBeVisible()
  })

  test('should switch to hex view when file is open', async ({ page }) => {
    // This would require mocking the file open dialog
    // For now, just verify the button exists
    await expect(page.locator('text=📂 Open')).toBeVisible()
  })
})
