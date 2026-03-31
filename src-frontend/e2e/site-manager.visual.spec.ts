import { test, expect } from '@playwright/test'

test.describe('Site Manager Dialog - Visual Regression', () => {
  test.beforeEach(async ({ page }) => {
    // Launch the dev server
    await page.goto('http://localhost:1420')
    
    // Wait for app to be ready
    await page.waitForSelector('.dynamic-menubar', { state: 'visible' })
  })

  test('Tools menu opens and shows submenu', async ({ page }) => {
    // Click Tools menu
    await page.getByRole('button', { name: 'Tools' }).click()
    
    // Wait for submenu animation
    await page.waitForTimeout(500)
    
    // Screenshot comparison - forces UI validation
    await expect(page).toHaveScreenshot('tools-menu-opened.png')
  })

  test('SFTP submenu expands on hover', async ({ page }) => {
    // Open Tools
    await page.getByRole('button', { name: 'Tools' }).click()
    await page.waitForTimeout(300)
    
    // Hover SFTP
    await page.getByRole('menuitem', { name: /SFTP/i }).hover()
    await page.waitForTimeout(500)
    
    // Screenshot comparison - forces submenu expansion validation
    await expect(page).toHaveScreenshot('sftp-submenu-expanded.png')
  })

  test('Site Manager dialog opens visually', async ({ page }) => {
    // Open Tools
    await page.getByRole('button', { name: 'Tools' }).click()
    await page.waitForTimeout(300)
    
    // Hover SFTP
    await page.getByRole('menuitem', { name: /SFTP/i }).hover()
    await page.waitForTimeout(500)
    
    // Click Site Manager
    await page.getByRole('menuitem', { name: /Site Manager/i }).click()
    
    // Wait for dialog animation
    await page.waitForTimeout(800)
    
    // Verify dialog appears
    const dialog = page.locator('.sftp-dialog')
    await expect(dialog).toBeVisible()
    
    // Screenshot comparison - forces dialog visibility validation
    await expect(page).toHaveScreenshot('site-manager-dialog-opened.png')
  })

  test('Add Site form appears visually', async ({ page }) => {
    // Open Site Manager
    await page.getByRole('button', { name: 'Tools' }).click()
    await page.getByRole('menuitem', { name: /SFTP/i }).hover()
    await page.getByRole('menuitem', { name: /Site Manager/i }).click()
    await page.waitForTimeout(800)
    
    // Click Add Site
    await page.getByRole('button', { name: /Add Site/i }).click()
    await page.waitForTimeout(500)
    
    // Verify form fields appear
    await expect(page.getByLabel(/Site Name/i)).toBeVisible()
    await expect(page.getByLabel(/Host/i)).toBeVisible()
    
    // Screenshot comparison
    await expect(page).toHaveScreenshot('add-site-form-visible.png')
  })

  test('Site saved appears in list visually', async ({ page }) => {
    // Open Site Manager
    await page.getByRole('button', { name: 'Tools' }).click()
    await page.getByRole('menuitem', { name: /SFTP/i }).hover()
    await page.getByRole('menuitem', { name: /Site Manager/i }).click()
    await page.waitForTimeout(800)
    
    // Add site
    await page.getByRole('button', { name: /Add Site/i }).click()
    await page.waitForTimeout(300)
    
    // Fill form
    await page.getByLabel(/Site Name/i).fill('Test Server')
    await page.getByLabel(/Host/i).fill('192.168.1.100')
    await page.getByLabel(/Username/i).fill('admin')
    
    // Save
    await page.getByRole('button', { name: /Add Site/i }).click()
    await page.waitForTimeout(500)
    
    // Verify site appears in list
    await expect(page.getByText('Test Server')).toBeVisible()
    
    // Screenshot comparison - forces site-in-list validation
    await expect(page).toHaveScreenshot('site-saved-in-list.png')
  })
})