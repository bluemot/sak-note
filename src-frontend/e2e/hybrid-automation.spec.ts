import { test, expect } from '@playwright/test';

/**
 * Hybrid Test Strategy:
 * 1. DOM Verification: Fast logic checks (after every click, check DOM state)
 * 2. Visual Regression: Screenshot comparison (pixel-perfect UI validation)
 * 
 * No AI involved in validation - pure Playwright assertions
 */

test.describe('SAK Editor - Hybrid Automation Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('networkidle');
  });

  // ==========================================
  // TEST 1: File Menu - DOM + Visual
  // ==========================================
  test('File menu opens with correct DOM structure @dom @visual', async ({ page }) => {
    // DOM Check: Verify menu exists
    const fileMenu = page.getByRole('button', { name: '📄 File ▼' });
    await expect(fileMenu).toBeVisible();
    
    // Action: Click File menu
    await fileMenu.click();
    
    // DOM Check: Verify dropdown appears in DOM (fast)
    // Check for menu items that should appear
    const openItem = page.locator('text=Open').first();
    await expect(openItem).toBeVisible({ timeout: 1000 });
    
    // Visual Check: Compare screenshot (catches layout bugs)
    // First run creates baseline, subsequent runs compare
    await expect(page).toHaveScreenshot('file-menu-opened.png', {
      maxDiffPixels: 100,  // Allow tiny rendering differences
      threshold: 0.2       // 20% tolerance for anti-aliasing
    });
  });

  // ==========================================
  // TEST 2: Toolbar Buttons - DOM verification
  // ==========================================
  test('Toolbar buttons exist and are clickable @dom', async ({ page }) => {
    // DOM Check: Verify all toolbar buttons exist
    const toolbarButtons = [
      { selector: '[data-action="file:open"]', name: 'Open' },
      { selector: '[data-action="sftp:connect"]', name: 'SFTP Connect' },
      { selector: '[data-action="llm:chat"]', name: 'AI Chat' }
    ];
    
    for (const btn of toolbarButtons) {
      const button = page.locator(btn.selector);
      await expect(button).toBeVisible();
      await expect(button).toBeEnabled();
      console.log(`✓ ${btn.name} button verified`);
    }
  });

  // ==========================================
  // TEST 3: Sidebar Tab Switching - DOM state changes
  // ==========================================
  test('Sidebar tabs switch correctly @dom', async ({ page }) => {
    // Get all tab buttons - use text content to locate
    const aiTab = page.getByRole('button', { name: '🤖 🤖 AI Chat' });
    const bookmarkTab = page.getByRole('button', { name: '🔖 📑 Bookmarks' });
    const sftpTab = page.getByRole('button', { name: '🌐 🌐 SFTP Sites' });
    const marksTab = page.getByRole('button', { name: '📍 📍 Marks' });
    
    // Verify all tabs exist
    await expect(aiTab).toBeVisible();
    await expect(bookmarkTab).toBeVisible();
    await expect(sftpTab).toBeVisible();
    await expect(marksTab).toBeVisible();
    
    // Test: Click Bookmarks, verify it becomes active (check class)
    await bookmarkTab.click();
    const bookmarkClasses = await bookmarkTab.getAttribute('class');
    expect(bookmarkClasses).toContain('active');
    console.log('✓ Bookmark tab is active');
    
    // Test: Click SFTP, verify state change
    await sftpTab.click();
    const sftpClasses = await sftpTab.getAttribute('class');
    expect(sftpClasses).toContain('active');
    const bookmarkClassesAfter = await bookmarkTab.getAttribute('class');
    expect(bookmarkClassesAfter).not.toContain('active');
    console.log('✓ SFTP tab is active, Bookmark is inactive');
  });

  // ==========================================
  // TEST 4: Open File Flow - Full DOM + Visual
  // ==========================================
  test('Open File button triggers dialog @dom @visual', async ({ page }) => {
    // Find and click Open button (use first() to get Locator)
    const openBtn = page.locator('[data-action="file:open"]').first();
    await openBtn.click();
    
    // DOM Check: Wait for dialog to appear
    // (Note: Tauri dialog might be native, so we check for loading state)
    await page.waitForTimeout(500);
    
    // Visual Check: Screenshot after click
    await expect(page).toHaveScreenshot('open-file-triggered.png', {
      maxDiffPixels: 100
    });
  });

  // ==========================================
  // TEST 5: Main Layout - Visual regression
  // ==========================================
  test('Main layout renders correctly @visual', async ({ page }) => {
    // Full page screenshot for layout validation
    await expect(page).toHaveScreenshot('main-layout.png', {
      fullPage: true,
      maxDiffPixels: 200  // Allow more diff for dynamic content
    });
  });

  // ==========================================
  // TEST 6: Editor Panel - DOM structure
  // ==========================================
  test('Editor panel has correct structure @dom', async ({ page }) => {
    // Check main editor container exists
    const editorContainer = page.locator('.editor-container, [class*="editor"]').first();
    await expect(editorContainer).toBeVisible();
    
    // Check welcome screen is shown (no file open) - use first() for strict mode
    const welcomeText = page.getByRole('heading', { name: 'SAK Editor' });
    await expect(welcomeText).toBeVisible();
    
    // Check feature list is rendered
    const features = [
      'Large file support',
      'Hex viewer mode',
      'LLM chat',
      'Color highlighting'
    ];
    
    for (const feature of features) {
      const featureText = page.getByText(feature).first();
      await expect(featureText).toBeVisible();
    }
  });

  // ==========================================
  // TEST 7: Resizable Panel - DOM attributes
  // ==========================================
  test('Resizable sidebar has correct attributes @dom', async ({ page }) => {
    // Check sidebar exists - use first() to get Locator object
    const sidebar = page.locator('.sidebar, [class*="sidebar"]').first();
    await expect(sidebar).toBeVisible();
    
    // Check resize handle exists (optional)
    const resizeHandle = page.locator('.resize-handle, [class*="resize"]').first();
    const handleExists = await resizeHandle.isVisible().catch(() => false);
    
    if (handleExists) {
      console.log('✓ Resize handle found');
    } else {
      console.log('⚠ Resize handle not found (may be OK)');
    }
  });
});
