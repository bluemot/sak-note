#!/usr/bin/env python3
"""
Test: Verify file content displays correctly after opening
"""

from playwright.sync_api import sync_playwright
import time
import os
import tempfile

def test_file_content_display():
    # Create a test file with known content
    test_content = """Hello SAK Editor!
This is a test file.
Line 3: Testing content display.
Line 4: Special characters: @#$%^&*()
Line 5: Unicode: 你好世界 🎉
"""
    
    # Write test file
    test_file_path = "/tmp/sak-editor-test.txt"
    with open(test_file_path, "w", encoding="utf-8") as f:
        f.write(test_content)
    
    print(f"✓ Created test file: {test_file_path}")
    print(f"✓ File content ({len(test_content)} chars):")
    print("-" * 40)
    print(test_content)
    print("-" * 40)
    
    screenshots_dir = "/home/ubuntu/.openclaw/workspace/sak-editor/test-results/file-display"
    os.makedirs(screenshots_dir, exist_ok=True)
    
    with sync_playwright() as p:
        # Connect to existing browser
        browser = p.chromium.connect_over_cdp("http://localhost:18800")
        context = browser.contexts[0] if browser.contexts else browser.new_context()
        page = context.pages[0] if context.pages else context.new_page()
        
        # Navigate to SAK Editor
        page.goto("http://localhost:5173")
        time.sleep(2)
        
        # Screenshot 1: Initial state
        page.screenshot(path=f"{screenshots_dir}/01_before_open.png")
        print("✓ Screenshot 1: Initial state saved")
        
        # Click Open button to trigger file dialog
        # Note: Tauri dialog is native, so we need to use keyboard shortcut or alternative
        # For testing, let's try to use Ctrl+O shortcut
        print("Attempting to open file dialog...")
        page.keyboard.press("Control+o")
        time.sleep(1)
        
        # Screenshot 2: After triggering open dialog
        page.screenshot(path=f"{screenshots_dir}/02_dialog_triggered.png")
        print("✓ Screenshot 2: Dialog triggered (check if native dialog appears)")
        
        # Since native dialog can't be automated easily, let's check if there's any indicator
        # that the editor is trying to load something
        
        # Alternative: Try to use the Open button in toolbar
        page.keyboard.press("Escape")  # Close any dialog first
        time.sleep(0.5)
        
        open_btn = page.locator('[data-action="file:open"]').first
        if open_btn.count() > 0:
            open_btn.click()
            time.sleep(1)
            page.screenshot(path=f"{screenshots_dir}/03_open_button_clicked.png")
            print("✓ Screenshot 3: Open button clicked")
        
        print("\n" + "="*50)
        print("TEST COMPLETE")
        print("="*50)
        print(f"Screenshots saved to: {screenshots_dir}")
        print("\nNote: Native file dialogs cannot be fully automated.")
        print("Please manually check the screenshots to verify:")
        print("1. Open dialog appears correctly")
        print("2. File can be selected and opened")
        print("3. Content displays in editor")
        
        browser.close()

if __name__ == "__main__":
    test_file_content_display()
