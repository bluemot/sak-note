# SAK Editor 檔案內容顯示測試

## 測試目的
驗證開檔後檔案內容可以正確顯示在 Editor 中

## 測試步驟

### 前置準備
1. 確保 dev server 正在運行: `cd src-frontend && npm run dev`
2. 確保 Tauri 已編譯: `cargo build` (in src-tauri)

### 測試 1: 基本文字檔案

**步驟:**
1. 啟動 SAK Editor: `cargo tauri dev` (from src-tauri)
2. 點擊 "Open File" 按鈕
3. 選擇 `/tmp/sak-editor-test.txt` (已自動建立)
4. 觀察 Editor 內容

**預期結果:**
- [ ] 檔案內容正確顯示：
  ```
  Hello SAK Editor!
  This is a test file.
  Line 3: Testing content display.
  Line 4: Special characters: @#$%^&*()
  Line 5: Unicode: 你好世界 🎉
  ```
- [ ] 行號正確
- [ ] 特殊字元正確顯示
- [ ] Unicode 字元正確顯示

### 測試 2: 大檔案 (>1MB)

**步驟:**
1. 產生大測試檔案:
   ```bash
   python3 -c "print('x' * 1000000)" > /tmp/large-test.txt
   ```
2. 在 SAK Editor 中開啟
3. 捲動檢查內容

**預期結果:**
- [ ] 大檔案能快速開啟 (不卡頓)
- [ ] 捲動流暢
- [ ] 記憶體使用合理

### 測試 3: 二進制檔案 (Hex Viewer)

**步驟:**
1. 開啟任意二進制檔案 (如圖片、PDF)
2. 切換到 Hex View

**預期結果:**
- [ ] Hex Viewer 正確顯示
- [ ] ASCII panel 同步顯示

## 已知問題

根據最近的 commit `9a7e4a8`: "Fix local file not displaying in Editor"
這個 fix 應該已經解決了本地檔案不顯示的問題。

## 自動化限制

Tauri 的 native dialog 無法被 Playwright 自動化，因此這個測試需要手動執行。

## 截圖記錄

測試過程中的截圖保存在 `test-results/file-display/` 目錄。
