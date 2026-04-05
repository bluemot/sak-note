#!/usr/bin/env python3
"""
Manual Browser Test for SAK Editor
每步操作後截圖，人工驗證結果
"""

from playwright.sync_api import sync_playwright
import time
import os

def test_sak_editor():
    screenshots_dir = "test-results/manual"
    os.makedirs(screenshots_dir, exist_ok=True)
    
    with sync_playwright() as p:
        # 啟動瀏覽器（連接到現有的 Chrome）
        browser = p.chromium.connect_over_cdp("http://localhost:18800")
        context = browser.contexts[0] if browser.contexts else browser.new_context()
        page = context.pages[0] if context.pages else context.new_page()
        
        # 導航到 SAK Editor
        page.goto("http://localhost:5173")
        time.sleep(2)
        
        # Step 1: 初始截圖
        page.screenshot(path=f"{screenshots_dir}/01_initial.png")
        print("✓ 已截圖: 01_initial.png")
        input("按 Enter 繼續下一步 (點擊 File Menu)...")
        
        # Step 2: 點擊 File Menu
        file_btn = page.locator('button:has-text("File")').first
        file_btn.click()
        time.sleep(0.5)
        page.screenshot(path=f"{screenshots_dir}/02_file_menu_clicked.png")
        print("✓ 已截圖: 02_file_menu_clicked.png")
        print("  請檢查截圖，確認 File menu 真的展開了")
        input("按 Enter 繼續下一步...")
        
        # Step 3: 點擊 Open 按鈕
        open_btn = page.locator('button:has-text("📂 Open")').first
        open_btn.click()
        time.sleep(0.5)
        page.screenshot(path=f"{screenshots_dir}/03_open_clicked.png")
        print("✓ 已截圖: 03_open_clicked.png")
        print("  請檢查截圖，確認 Open 對話框出現")
        input("按 Enter 繼續下一步...")
        
        # Step 4: 點擊 Bookmarks tab
        bookmark_btn = page.locator('button:has-text("Bookmarks")').first
        bookmark_btn.click()
        time.sleep(0.5)
        page.screenshot(path=f"{screenshots_dir}/04_bookmarks_tab.png")
        print("✓ 已截圖: 04_bookmarks_tab.png")
        print("  請檢查截圖，確認 Bookmarks panel 顯示")
        input("按 Enter 繼續下一步...")
        
        # Step 5: 點擊 SFTP Sites tab
        sftp_btn = page.locator('button:has-text("SFTP Sites")').first
        sftp_btn.click()
        time.sleep(0.5)
        page.screenshot(path=f"{screenshots_dir}/05_sftp_tab.png")
        print("✓ 已截圖: 05_sftp_tab.png")
        print("  請檢查截圖，確認 SFTP Sites panel 顯示")
        
        print("\n✅ 所有步驟完成！請查看 screenshots 目錄確認結果")
        
        browser.close()

if __name__ == "__main__":
    test_sak_editor()
