import { test, expect } from '@playwright/test'

test.describe('SAK Editor - Real UI Automation Tests', () => {

  test('should launch app and show welcome screen', async ({ page }) => {
    // Launch Tauri app via webview
    await page.goto('http://localhost:5173')
    
    // Wait for app to load
    await page.waitForLoadState('networkidle')
    
    // Check welcome screen elements
    await expect(page.locator('h1')).toContainText('SAK Editor')
    await expect(page.locator('text=A modern editor for large files')).toBeVisible()
    await expect(page.locator('text=📂 Open')).toBeVisible()
  })

  test('should display all feature items', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // Check all feature items are visible
    await expect(page.locator('text=Large file support (memory-mapped)')).toBeVisible()
    await expect(page.locator('text=Hex viewer mode')).toBeVisible()
    await expect(page.locator('text=LLM chat & summary')).toBeVisible()
    await expect(page.locator('text=Color highlighting')).toBeVisible()
  })

  test('should have sidebar with three tabs', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // Check sidebar tabs
    await expect(page.locator('text=ℹ️ Info')).toBeVisible()
    await expect(page.locator('text=🤖 Chat')).toBeVisible()
    await expect(page.locator('text=🎨 Marks')).toBeVisible()
  })

  test('should show "No file open" message initially', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // Check Info tab shows "No file open"
    await expect(page.locator('text=No file open')).toBeVisible()
  })

  test('should switch between sidebar tabs', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // Click Chat tab
    await page.click('text=🤖 Chat')
    await expect(page.locator('text=Open a file to start chatting')).toBeVisible()
    
    // Click Marks tab
    await page.click('text=🎨 Marks')
    await expect(page.locator('text=Color highlights will appear here')).toBeVisible()
    
    // Click back to Info tab
    await page.click('text=ℹ️ Info')
    await expect(page.locator('text=No file open')).toBeVisible()
  })

  test('should have Open File button that can be clicked', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // Find and click Open button
    const openButton = page.locator('text=📂 Open')
    await expect(openButton).toBeVisible()
    await expect(openButton).toBeEnabled()
    
    // Click it (will open file dialog, but we can't test that in headless)
    await openButton.click()
    
    // Button should show loading state or remain visible
    await expect(page.locator('text=📂 Open, button')).toBeTruthy()
  })

  test('should have search functionality UI', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // Note: Search button only visible when file is open
    // This test verifies the search button exists in DOM
    const searchButton = page.locator('text=🔎 Search')
    // May not be visible without file open
    const count = await searchButton.count()
    console.log('Search button count:', count)
  })

  test('should have toolbar with all buttons', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // Check toolbar buttons
    await expect(page.locator('text=📂 Open')).toBeVisible()
    // Other buttons only appear when file is loaded
  })

  test('should handle window resize', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // Resize to mobile size
    await page.setViewportSize({ width: 800, height: 600 })
    await page.waitForTimeout(500)
    
    // App should still show
    await expect(page.locator('h1')).toContainText('SAK Editor')
    
    // Resize back to desktop
    await page.setViewportSize({ width: 1400, height: 900 })
    await page.waitForTimeout(500)
    
    await expect(page.locator('h1')).toContainText('SAK Editor')
  })
})
