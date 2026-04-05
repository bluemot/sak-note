import { test, expect } from '@playwright/test'

test.describe('Site Manager Dialog - Visual Regression', () => {
  test.beforeEach(async ({ page }) => {
    // Launch the dev server
    await page.goto('http://localhost:5173')
    
    // Wait for app to be ready
    await page.waitForSelector('.dynamic-menubar', { state: 'visible' })
  })

  test('Tools menu opens and shows submenu', async ({ page }) => {
    // Click Tools menu
    await page.getByRole('button', { name: 'Tools' }).click()
    
    // Wait for submenu animation
    await page.waitForTimeout(500)
    
    // Verify SFTP submenu exists
    const sftpText = page.getByText(/SFTP/i).first()
    await expect(sftpText).toBeVisible()
  })

  test('SFTP submenu expands on hover', async ({ page }) => {
    // Open Tools
    await page.getByRole('button', { name: 'Tools' }).click()
    await page.waitForTimeout(300)
    
    // Hover SFTP - use text locator with first() to avoid strict mode
    const sftpMenuItem = page.getByText(/SFTP/i).first()
    await expect(sftpMenuItem).toBeVisible()
    await sftpMenuItem.hover()
    await page.waitForTimeout(500)
    
    // Verify submenu items appear
    const siteManagerItem = page.getByText(/Site Manager/i).first()
    await expect(siteManagerItem).toBeVisible()
  })

  test('Site Manager dialog opens visually', async ({ page }) => {
    // Open Tools
    await page.getByRole('button', { name: 'Tools' }).click()
    await page.waitForTimeout(300)
    
    // Hover SFTP - use text locator with first()
    const sftpMenuItem = page.getByText(/SFTP/i).first()
    await expect(sftpMenuItem).toBeVisible()
    await sftpMenuItem.hover()
    await page.waitForTimeout(500)
    
    // Click Site Manager - use text locator with first()
    const siteManagerItem = page.getByText(/Site Manager/i).first()
    await expect(siteManagerItem).toBeVisible()
    await siteManagerItem.click()
    
    // Wait for dialog animation
    await page.waitForTimeout(800)
    
    // Verify dialog appears
    const dialog = page.locator('.sftp-dialog')
    await expect(dialog).toBeVisible()
    
    console.log('Site Manager dialog opened successfully')
  })

  test('Add Site form appears visually', async ({ page }) => {
    // Open Site Manager
    await page.getByRole('button', { name: 'Tools' }).click()
    const sftpMenuItem = page.getByText(/SFTP/i).first()
    await expect(sftpMenuItem).toBeVisible()
    await sftpMenuItem.hover()
    const siteManagerItem = page.getByText(/Site Manager/i).first()
    await expect(siteManagerItem).toBeVisible()
    await siteManagerItem.click()
    await page.waitForTimeout(800)
    
    // Verify dialog is open
    const dialog = page.locator('.sftp-dialog')
    await expect(dialog).toBeVisible()
    
    // Click Add Site - use first() for locator
    const addSiteBtn = page.getByRole('button', { name: /Add Site/i }).first()
    await expect(addSiteBtn).toBeVisible()
    await addSiteBtn.click()
    await page.waitForTimeout(500)
    
    // Verify form fields appear - use first() for locators
    // Use input with placeholder or text containing the field name
    const siteNameInput = page.locator('input, textarea').filter({ hasText: /Site Name|Host|Username|Password/i }).first()
      .or(page.locator('input[placeholder*="Site"], input[placeholder*="Name"]').first())
      .or(page.locator('.sftp-dialog input').first())
    
    // Check if any form input exists
    const anyInput = page.locator('.sftp-dialog input, .sftp-dialog form input').first()
    await expect(anyInput).toBeVisible()
    
    console.log('Add Site form appears correctly')
  })

  test('Site saved appears in list visually', async ({ page }) => {
    // Open Site Manager
    await page.getByRole('button', { name: 'Tools' }).click()
    const sftpMenuItem = page.getByText(/SFTP/i).first()
    await expect(sftpMenuItem).toBeVisible()
    await sftpMenuItem.hover()
    const siteManagerItem = page.getByText(/Site Manager/i).first()
    await expect(siteManagerItem).toBeVisible()
    await siteManagerItem.click()
    await page.waitForTimeout(800)
    
    // Verify dialog is open
    const dialog = page.locator('.sftp-dialog')
    await expect(dialog).toBeVisible()
    
    // Click Add Site
    const addSiteBtn = page.getByRole('button', { name: /Add Site/i }).first()
    await expect(addSiteBtn).toBeVisible()
    await addSiteBtn.click()
    await page.waitForTimeout(300)
    
    // Fill form - get all text inputs and fill them (skip number inputs like port)
    const textInputs = page.locator('.sftp-dialog input:not([type="number"])')
    const inputCount = await textInputs.count()
    
    // Fill only text inputs (site name, host, username, password)
    const fillValues = ['Test Server', '192.168.1.100', 'admin']
    for (let i = 0; i < Math.min(inputCount, fillValues.length); i++) {
      await textInputs.nth(i).fill(fillValues[i])
    }
    
    // Save
    const saveBtn = page.getByRole('button', { name: /Add Site|Save/i }).first()
    await saveBtn.click()
    await page.waitForTimeout(500)
    
    // Verify site appears in list
    await expect(page.getByText('Test Server').first()).toBeVisible()
    
    console.log('Site saved successfully')
  })
})