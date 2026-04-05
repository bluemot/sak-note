#!/usr/bin/env python3
"""
Auto Browser Test for SAK Editor
自動執行所有步驟，每步都截圖
"""

from playwright.sync_api import sync_playwright
import time
import os

def test_sak_editor():
    screenshots_dir = "/home/ubuntu/.openclaw/workspace/sak-editor/test-results/manual"
    os.makedirs(screenshots_dir, exist_ok=True)
    
    with sync_playwright() as p:
        # 啟動瀏覽器（連接到現有的 Chrome）
        browser = p.chromium.connect_over_cdp("http://localhost:18800")
        context = browser.contexts[0] if browser.contexts else browser.new_context()
        page = context.pages[0] if context.pages else context.new_page()
        
        print("🔗 已連接到瀏覽器")
        print(f"📁 截圖將保存到: {screenshots_dir}")
        print()
        
        # Step 1: 初始截圖
        print("Step 1: 載入初始頁面...")
        page.goto("http://localhost:5173")
        time.sleep(2)
        page.screenshot(path=f"{screenshots_dir}/01_initial.png")
        print(f"  ✓ 已截圖: 01_initial.png")
        
        # Step 2: 點擊 File Menu
        print("Step 2: 點擊 File Menu...")
        file_btn = page.locator('button:has-text("📄 File ▼")').first
        file_btn.click()
        time.sleep(0.5)
        page.screenshot(path=f"{screenshots_dir}/02_file_menu_clicked.png")
        print(f"  ✓ 已截圖: 02_file_menu_clicked.png")
        
        # Step 3: 點擊 Open 按�（在 Toolbar）
        print("Step 3: 點擊 Toolbar Open 按�...")
        # 先關閉可能打開的 menu
        page.keyboard.press("Escape")
        time.sleep(0.3)
        
        open_btn = page.locator('[data-action="file:open"]').first
        open_btn.click()
        time.sleep(0.5)
        page.screenshot(path=f"{screenshots_dir}/03_open_clicked.png")
        print(f"  ✓ 已截圖: 03_open_clicked.png")
        
        # Step 4: 關閉 dialog（如果有的話），然後點擊 Bookmarks tab
        print("Step 4: 點擊 Bookmarks Tab...")
        page.keyboard.press("Escape")
        time.sleep(0.3)
        
        bookmark_btn = page.locator('button:has-text("📑 Bookmarks")').first
        bookmark_btn.click()
        time.sleep(0.5)
        page.screenshot(path=f"{screenshots_dir}/04_bookmarks_tab.png")
        print(f"  ✓ 已截圖: 04_bookmarks_tab.png")
        
        # Step 5: 點擊 SFTP Sites tab
        print("Step 5: 點擊 SFTP Sites Tab...")
        sftp_btn = page.locator('button:has-text("🌐 SFTP Sites")').first
        sftp_btn.click()
        time.sleep(0.5)
        page.screenshot(path=f"{screenshots_dir}/05_sftp_tab.png")
        print(f"  ✓ 已截圖: 05_sftp_tab.png")
        
        # Step 6: 點擊 AI Chat tab
        print("Step 6: 點擊 AI Chat Tab...")
        ai_btn = page.locator('button:has-text("🤖 AI Chat")').first
        ai_btn.click()
        time.sleep(0.5)
        page.screenshot(path=f"{screenshots_dir}/06_ai_chat_tab.png")
        print(f"  ✓ 已截圖: 06_ai_chat_tab.png")
        
        print()
        print("=" * 50)
        print("✅ 所有步驟完成！")
        print()
        print("請查看以下截圖來驗證結果:")
        for i in range(1, 7):
            print(f"  - test-results/manual/{i:02d}_*.png")
        print()
        
        browser.close()

if __name__ == "__main__":
    test_sak_editor()
