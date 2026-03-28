# SAK Editor - E2E 功能測試計畫

## 測試執行方式
```bash
# 完整測試
./scripts/e2e-test-suite.sh

# 單一類別測試
./scripts/e2e-test-suite.sh --category=file
./scripts/e2e-test-suite.sh --category=edit
```

---

## ✅ 可以測試的功能 (UI + xdotool)

### 1. File Operations (檔案操作)
| # | 測試項目 | 方法 | 驗證方式 |
|---|---------|------|---------|
| F01 | Open File Dialog (Ctrl+O) | xdotool key ctrl+o | 對話框出現 |
| F02 | Save Dialog (Ctrl+Shift+S) | xdotool key ctrl+shift+s | 對話框出現 |
| F03 | New File (Ctrl+N) | xdotool key ctrl+n | UI 狀態變化 |
| F04 | Close File (Ctrl+W) | xdotool key ctrl+w | 回到 WelcomeScreen |
| F05 | Recent Files Menu | 點擊 File → Open Recent | 子選單出現 |
| F06 | Save Button State | 檢查 disabled/enabled | 條件渲染驗證 |

### 2. Edit Operations (編輯操作)
| # | 測試項目 | 方法 | 驗證方式 |
|---|---------|------|---------|
| E01 | Undo (Ctrl+Z) | xdotool key ctrl+z | 狀態列顯示 |
| E02 | Redo (Ctrl+Y) | xdotool key ctrl+y | 狀態列顯示 |
| E03 | Find (Ctrl+F) | xdotool key ctrl+f | SearchPanel 出現 |
| E04 | Replace (Ctrl+H) | xdotool key ctrl+h | Replace dialog |
| E05 | Go to Line (Ctrl+G) | xdotool key ctrl+g | Goto dialog |
| E06 | Find in Files (Ctrl+Shift+F) | xdotool key ctrl+shift+f | 全局搜尋 dialog |
| E07 | Cut (Ctrl+X) | xdotool key ctrl+x | clipboard 狀態 |
| E08 | Copy (Ctrl+C) | xdotool key ctrl+c | clipboard 狀態 |
| E09 | Paste (Ctrl+V) | xdotool key ctrl+v | 內容插入 |

### 3. UI Components (UI 組件)
| # | 測試項目 | 方法 | 驗證方式 |
|---|---------|------|---------|
| U01 | MenuBar 顯示 | 視覺檢查 | File/Edit/View/Tools/Plugins |
| U02 | Toolbar 顯示 | 視覺檢查 | 按鈕存在 |
| U03 | Sidebar Tabs | 點擊各 tab | 內容切換 |
| U04 | StatusBar 顯示 | 視覺檢查 | 底部狀態 |
| U05 | Resizable Sidebar | 拖曳邊界 | 寬度改變 |
| U06 | Notification Toast | 觸發通知 | 右上角顯示 |
| U07 | Search Panel | Ctrl+F 展開 | 底部面板 |

### 4. View Operations (檢視操作)
| # | 測試項目 | 方法 | 驗證方式 |
|---|---------|------|---------|
| V01 | Toggle Sidebar | Ctrl+B 或選單 | Sidebar 顯示/隱藏 |
| V02 | Toggle StatusBar | 選單操作 | StatusBar 顯示/隱藏 |
| V03 | Fullscreen | F11 | 視窗全螢幕 |
| V04 | Zoom In (Ctrl++) | xdotool key ctrl+plus | 字體放大 |
| V05 | Zoom Out (Ctrl+-) | xdotool key ctrl+minus | 字體縮小 |

### 5. Navigation (導航)
| # | 測試項目 | 方法 | 驗證方式 |
|---|---------|------|---------|
| N01 | Next Bookmark (F2) | xdotool key F2 | 跳轉 |
| N02 | Prev Bookmark (Shift+F2) | xdotool key shift+F2 | 跳轉 |
| N03 | Next Mark | 選單或快捷鍵 | 跳轉 |
| N04 | Prev Mark | 選單或快捷鍵 | 跳轉 |

### 6. LLM Features (AI 功能)
| # | 測試項目 | 方法 | 驗證方式 |
|---|---------|------|---------|
| L01 | Chat Panel 開啟 | 點擊 Chat tab | 面板顯示 |
| L02 | Summarize | 選單操作 | 請求發送 |
| L03 | AI Settings | 選單操作 | Settings dialog |

---

## ⚠️ 需要模擬/間接測試的功能

### 1. File Content Operations
| # | 測試項目 | 限制 | 替代方案 |
|---|---------|------|---------|
| C01 | 實際檔案開啟 | 需要真實檔案 | 建立測試檔案 |
| C02 | 內容編輯 | Monaco Editor 難以自動化 | 使用 Monaco API |
| C03 | 儲存檔案內容 | 需要檔案系統權限 | 驗證 dialog 出現 |
| C04 | 搜尋結果高亮 | 需要內容 | 驗證 panel 顯示 |

### 2. SFTP Operations (遠端操作)
| # | 測試項目 | 限制 | 替代方案 |
|---|---------|------|---------|
| S01 | SFTP Connect | 需要伺服器 | Mock 或 skip |
| S02 | Open Remote File | 需要連線 | 驗證 dialog |
| S03 | Site Manager | UI 測試 | 開啟 dialog |
| S04 | Disconnect | UI 測試 | 按鈕狀態 |

---

## ❌ 無法測試的功能 (需要特殊環境)

### 1. System Integration (系統整合)
| # | 測試項目 | 原因 | 註解 |
|---|---------|------|------|
| X01 | 實際檔案 I/O | 需要真實檔案系統 | 可用 tmpfs 測試 |
| X02 | 剪貼簿操作 | 需要 X11 clipboard | xdotool 可驗證 |
| X03 | 列印功能 | 需要印表機 | 驗證 Print dialog |
| X04 | PDF Export | 需要 PDF 函式庫 | 驗證 dialog |

### 2. Plugin System (外掛系統)
| # | 測試項目 | 原因 | 註解 |
|---|---------|------|------|
| P01 | WASM Plugin 載入 | 需要實際 .wasm 檔案 | 建立測試 plugin |
| P02 | Plugin UI 註冊 | 需要載入 plugin | 驗證註冊流程 |
| P03 | Plugin 事件處理 | 需要 runtime | Mock 測試 |

### 3. Network Operations (網路操作)
| # | 測試項目 | 原因 | 註解 |
|---|---------|------|------|
| N01 | LLM API 呼叫 | 需要網路/API key | Mock 或 skip |
| N02 | SFTP 傳輸 | 需要 SSH server | 使用測試容器 |
| N03 | Auto-update | 需要版本伺服器 | Skip |

### 4. Performance Tests (性能測試)
| # | 測試項目 | 原因 | 註解 |
|---|---------|------|------|
| T01 | 大檔案開啟 (>1GB) | 需要大檔案 | 產生測試檔案 |
| T02 | Memory-mapped I/O | 需要觀察記憶體 | 系統工具 |
| T03 | 搜尋速度 | 需要大檔案 | 計時測試 |

---

## 📝 測試實作建議

### 優先級 1: 核心功能 (必須有)
```bash
# 檔案操作
F01, F02, F03, F04, F05

# 編輯操作
E01, E02, E03, E04, E05

# UI 組件
U01, U02, U03, U04
```

### 優先級 2: 重要功能 (建議有)
```bash
# 快捷鍵
E06, E07, E08, E09
V01, V02, V04, V05

# 導航
N01, N02
```

### 優先級 3: 進階功能 (有時間再加)
```bash
# LLM
L01, L02, L03

# 外掛
P01, P02

# 性能
T01 (小檔案)
```

---

## 🔧 測試工具

### xdotool 常用指令
```bash
# 視窗操作
xdotool search --name "SAK Editor"          # 找視窗
xdotool windowactivate <id>                   # 啟動視窗
xdotool windowfocus <id>                      # 聚焦
xdotool windowsize <id> 1400 900              # 調整大小

# 滑鼠操作
xdotool mousemove --window <id> 100 100       # 移動滑鼠
xdotool click 1                               # 左鍵點擊
xdotool mousedown 1 / mouseup 1               # 拖曳

# 鍵盤操作
xdotool key ctrl+o                            # 快捷鍵
xdotool type "hello"                          # 輸入文字
xdotool key Escape                            # 單一按鍵
```

### 截圖驗證
```bash
# 視窗截圖
import -window <id> /tmp/screenshot.png

# 比對截圖 (需要 imagemagick)
compare screenshot1.png screenshot2.png diff.png
```

---

## 📊 測試報告格式

```json
{
  "test_run": "2026-03-28T20:00:00+08:00",
  "total_tests": 35,
  "passed": 33,
  "failed": 2,
  "skipped": 0,
  "categories": {
    "file": { "total": 6, "passed": 6, "failed": 0 },
    "edit": { "total": 9, "passed": 8, "failed": 1 },
    "ui": { "total": 7, "passed": 7, "failed": 0 },
    "view": { "total": 5, "passed": 4, "failed": 1 },
    "navigation": { "total": 4, "passed": 4, "failed": 0 },
    "llm": { "total": 3, "passed": 3, "failed": 0 },
    "sftp": { "total": 4, "passed": 1, "failed": 0, "skipped": 3 }
  },
  "duration_seconds": 120
}
```

---

## 🎯 執行計畫

### Phase 1: 基礎測試 (已完成 ✅)
- [x] Window Launch
- [x] MenuBar, Toolbar, Sidebar
- [x] Basic keyboard shortcuts

### Phase 2: 核心功能測試 (建議)
- [ ] File operations (Open/Save dialogs)
- [ ] Edit operations (Undo/Redo/Find)
- [ ] View operations (Sidebar toggle)

### Phase 3: 完整功能測試 (未來)
- [ ] SFTP UI (mock)
- [ ] LLM panel
- [ ] Plugin system
- [ ] Performance (small files)

---

**Updated**: 2026-03-28
**Author**: Jarvis
**Status**: Phase 1 Complete, Phase 2 Planned
