# Windows Build Fix - Icon Format Error

## 錯誤訊息
```
error RC2175 : resource file icons\icon.ico is not in 3.00 format
```

## 原因
Tauri 需要的 `.ico` 檔案必須是特定格式的 Windows 圖示檔案。

## 解決方案

### 方案 1: 暫時移除 ICO（已套用）
修改 `src-tauri/tauri.conf.json`，移除 `icons/icon.ico`：
```json
"icon": [
  "icons/32x32.png",
  "icons/128x128.png",
  "icons/128x128@2x.png",
  "icons/icon.icns"
  // 移除: "icons/icon.ico"
],
```

### 方案 2: 生成正確的 ICO（推薦）

在你的 Windows 機器上執行：

```powershell
# 安裝 imagemagick（如果還沒有）
# https://imagemagick.org/script/download.php#windows

# 從 PNG 生成正確的 ICO
cd D:\workspace\sak-note\src-tauri\icons
magick convert 128x128.png -define icon:auto-resize=128,64,48,32,16 icon.ico

# 或線上工具
# https://convertio.co/png-ico/
# https://icoconvert.com/
```

### 方案 3: 使用 Tauri 官方圖示

```powershell
cd D:\workspace\sak-note\src-tauri
# 備份並替換
copy icons\128x128.png icons\icon-temp.png
# 下載預設圖示或使用線上轉換工具
```

## 完整修復步驟

1. **暫時方案**（立即生效）：
   - 我已經修改 `tauri.conf.json` 移除 ICO
   - 重新執行 `build.bat`

2. **永久方案**（之後補上）：
   - 生成正確的 `icon.ico`
   - 放進 `src-tauri/icons/`
   - 恢復 `tauri.conf.json` 中的 `"icons/icon.ico"`

## 測試

修改後請重新執行：
```powershell
cd D:\workspace\sak-note
build.bat
```

## 參考

- Tauri Issue: https://github.com/tauri-apps/tauri/issues/5184
- 正確的 ICO 需要包含多種解析度（16x16, 32x32, 48x48, 128x128, 256x256）
