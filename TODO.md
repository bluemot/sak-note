# SAK Editor - 待實現功能清單 (TODO List)

**建立日期**: 2026-03-28
**專案狀態**: 60+ 底層功能完成，UI 整合度低

---

## 🚨 重大缺失：基礎 UI 組件

### 1. Menu Bar (選單欄)
**目前狀態**: ❌ 完全不存在
**影響**: 大量底層功能無入口

**需要實現**:
- [ ] **MenuBar.tsx** - 頂層選單欄主組件
- [ ] **MenuItem.tsx** - 選單項目（支援子選單、快捷鍵顯示）
- [ ] **Menu.tsx** - 下拉選單容器

**選單結構**:
```
File Menu
├── Open (Ctrl+O)
├── Open Remote (SFTP) ⭐高優先
├── Save (Ctrl+S) ⭐高優先
├── Save As (Ctrl+Shift+S) ⭐高優先
├── Recent Files → ⭐中優先
├── Save Session ⭐中優先
├── Load Session ⭐中優先
├── Print (Ctrl+P) ⭐中優先
├── Export to PDF ⭐中優先
└── Exit

Edit Menu
├── Undo (Ctrl+Z) ⭐高優先
├── Redo (Ctrl+Y) ⭐高優先
├── Cut (Ctrl+X) ⭐高優先（需補充底層）
├── Copy (Ctrl+C) ⭐高優先（需補充底層）
├── Paste (Ctrl+V) ⭐高優先（需補充底層）
├── Separator
├── Find (Ctrl+F)
├── Replace (Ctrl+H)
├── Find in Files (Ctrl+Shift+F)
├── Go to Line (Ctrl+G)
├── Separator
├── Bookmarks → ⭐中優先
├── Marks → ⭐中優先
└── Line Operations → ⭐低優先

View Menu
├── Text View
├── Hex View
├── Separator
├── Sidebar (顯示/隱藏)
├── Toolbar (顯示/隱藏)
├── Status Bar (顯示/隱藏) ⭐高優先
├── Separator
├── Zoom In (Ctrl++)
├── Zoom Out (Ctrl+-)
└── Full Screen (F11)

Settings Menu
├── AI Server Settings ⭐高優先
├── Preferences
└── Keyboard Shortcuts

Plugins Menu
├── Plugin Manager ⭐中優先
└── [動態外掛選單]
```

---

### 2. Status Bar (狀態欄)
**目前狀態**: ❌ 完全不存在
**影響**: 用戶無法看到檔案狀態、游標位置

**需要實現**:
- [ ] **StatusBar.tsx** - 底層狀態欄組件

**顯示內容**:
```
[行號, 列號] | [編碼: UTF-8] | [檔案格式: Unix/Windows] | [未儲存 *] | [進度: Ready]
```

**功能**:
- [ ] 顯示目前游標行號/列號
- [ ] 顯示檔案編碼 (UTF-8/GBK/Big5)
- [ ] 顯示行結尾格式 (LF/CRLF)
- [ ] 顯示儲存狀態 (* 未儲存)
- [ ] 顯示檔案大小
- [ ] 顯示目前模式 (Insert/Overwrite)

---

## 🔴 高優先級：功能入口缺失

### 3. Save / Save As 功能入口
**底層**: ✅ `save_file`, `save_as` commands 已實現
**目前**: ❌ Toolbar 沒有 Save 按鈕，沒有 File 選單
**工時估計**: 2-4 小時

**待實現**:
- [ ] Toolbar Save 按鈕 (💾 圖示)
- [ ] Toolbar Save As 按鈕
- [ ] Menu: File → Save
- [ ] Menu: File → Save As
- [ ] Keyboard: Ctrl+S
- [ ] Keyboard: Ctrl+Shift+S
- [ ] 未儲存關閉確認對話框
- [ ] 標題欄顯示未儲存標記 (*)

---

### 4. Search Panel (搜尋結果下方欄) ⭐新增
**底層**: ✅ Search engine (Boyer-Moore-Horspool) 已實現
**目前**: ❌ 只有簡單搜尋框，沒有結果顯示面板
**工時估計**: 6-8 小時

**待實現**:
- [ ] **SearchPanel.tsx** - 搜尋結果下方欄組件
- [ ] Menu: Search → Find in Current File (Ctrl+F)
- [ ] 搜尋輸入框 + 選項 (Regex/Case sensitive/Whole word)
- [ ] 結果列表顯示：行號 + 內容 + 高亮匹配文字
- [ ] 雙擊結果跳轉到該行
- [ ] 顯示結果數量 (e.g., "5/12 matches")
- [ ] 上一個/下一個結果按鈕 (F3/Shift+F3)
- [ ] 關閉搜尋面板按鈕 (Escape)
- [ ] 搜尋歷史下拉選單

**搜尋結果格式**:
```
Line 42:    function helloWorld() {      <-- 匹配結果
Line 55:    const hello = "world";       <-- 匹配結果
Line 128:   helloWorld();                <-- 匹配結果
```

---

### 5. Resizable Panels (可調整大小面板) ⭐新增
**底層**: N/A (純前端功能)
**目前**: ❌ Sidebar 和 Search Panel 固定大小
**工時估計**: 4-6 小時

**待實現**:
- [ ] **ResizableContainer.tsx** - 可拖曳調整大小的容器
- [ ] Sidebar (左側) 可拖曳右邊界調整寬度
- [ ] Search Panel (下方) 可拖曳上邊界調整高度
- [ ] 最小/最大寬度/高度限制
- [ ] 記住用戶調整後的大小（持續化）
- [ ] 雙擊收起/展開面板
- [ ] 拖曳時顯示實時預覽

**布局架構**:
```
┌─────────────────────────────────────┐
│  Menu Bar                           │
├─────────────────────────────────────┤
│  Toolbar                            │
├──────┬──────────────────────────────┤
│      │                              │
│ Side │      Editor                  │
│ bar  │      (可調整區域)             │
│ (可  │                              │
│ 拖曳)├───────────────────────────────┤
│      │      Search Results Panel    │
│      │      (可拖曳調整高度)          │
└──────┴──────────────────────────────┘
       ↑                    ↑
   Resizable           Resizable
   Handle              Handle
```

---

### 6. SFTP 遠端檔案入口
**底層**: ✅ SftpModule 完整
**目前**: ⚠️ SftpSiteManager.tsx 存在但沒入口
**工時估計**: 4-6 小時

**待實現**:
- [ ] Menu: File → Open Remote (SFTP)
- [ ] Menu: File → SFTP Site Manager
- [ ] Toolbar Open Remote 按鈕 (🌐)
- [ ] 最近使用的 SFTP 站台
- [ ] 側邊欄 Remote tab 整合

---

### 5. AI Server 設定介面
**底層**: ✅ LlmModule 已連線
**目前**: ❌ 完全沒有設定介面
**工時估計**: 4-6 小時

**待實現**:
- [ ] Menu: Settings → AI Server Settings
- [ ] AISettingsDialog.tsx
- [ ] Ollama Server URL 輸入
- [ ] API Key 輸入 (選填)
- [ ] 預設模型選擇下拉選單
- [ ] Temperature slider (0.0-2.0)
- [ ] Max Tokens 輸入
- [ ] 連線測試按鈕
- [ ] 測試結果顯示

---

### 6. Edit 選單基礎功能 (Cut/Copy/Paste)
**底層**: ⚠️ 需要補充 commands
**目前**: ❌ 完全沒有 Edit 選單
**工時估計**: 6-8 小時（含底層實現）

**待實現**:
- [ ] Rust: `clipboard_cut` command
- [ ] Rust: `clipboard_copy` command
- [ ] Rust: `clipboard_paste` command
- [ ] Menu: Edit → Cut (Ctrl+X)
- [ ] Menu: Edit → Copy (Ctrl+C)
- [ ] Menu: Edit → Paste (Ctrl+V)
- [ ] Toolbar Cut/Copy/Paste 按鈕

---

## 🟡 中優先級：功能連結待驗證

### 7. Color Marks 功能驗證
**底層**: ✅ MarksModule 完整
**目前**: ⚠️ MarkPanel 存在，連結待確認
**工時估計**: 4-6 小時

**待實現**:
- [ ] 驗證：Create mark → 呼叫 `create_mark`
- [ ] Editor highlight 顯示標記
- [ ] 右鍵選單：Add Mark
- [ ] 右鍵選單：Remove Mark
- [ ] Keyboard: Ctrl+Shift+1~9 快速標記
- [ ] Mark Panel 點擊跳轉

---

### 8. Session Management 整合
**底層**: ✅ session_save/load/delete 已實現
**目前**: ⚠️ SessionManager.tsx 存在
**工時估計**: 3-4 小時

**待實現**:
- [ ] Menu: File → Save Session
- [ ] Menu: File → Load Session
- [ ] Menu: File → Recent Sessions
- [ ] 啟動時恢復工作階段選項
- [ ] 側邊欄 Sessions tab

---

### 9. Bookmark 功能強化
**底層**: ✅ BookmarkModule 已實現
**目前**: ⚠️ BookmarkPanel 存在
**工時估計**: 3-4 小時

**待實現**:
- [ ] Keyboard: Ctrl+F2 Toggle bookmark
- [ ] Keyboard: F2 Next bookmark
- [ ] Keyboard: Shift+F2 Previous bookmark
- [ ] Editor gutter 顯示 bookmark 圖示
- [ ] Menu: Edit → Bookmarks 子選單

---

### 10. Print / Export PDF
**底層**: ✅ Print Manager 已實現
**目前**: ⚠️ 可能沒有入口
**工時估計**: 2-3 小時

**待實現**:
- [ ] Menu: File → Print (Ctrl+P)
- [ ] Menu: File → Export to PDF
- [ ] Toolbar Print 按鈕 (🖨️)
- [ ] Print 對話框整合

---

### 11. Plugin Manager UI
**底層**: ✅ WASM Plugin System 完整
**目前**: ⚠️ 沒有管理介面
**工時估計**: 6-8 小時

**待實現**:
- [ ] Menu: Plugins → Plugin Manager
- [ ] PluginManagerDialog.tsx
- [ ] 已安裝外掛列表
- [ ] 安裝/移除外掛按鈕
- [ ] 啟用/停用外掛開關
- [ ] 外掛設定編輯

---

## 🟢 低優先級：體驗增強

### 12. Find in Files 增強
**底層**: ✅ FindInFiles 模組已實現
**目前**: ✅ FindInFiles.tsx 存在
**工時估計**: 2-3 小時

**待實現**:
- [ ] 結果雙擊開啟檔案
- [ ] 搜尋進度條
- [ ] 結果匯出功能
- [ ] Regex 語法檢查

---

### 13. Recent Files 增強
**底層**: ✅ Recent Files 列表持續化
**目前**: ✅ RecentFiles.tsx 存在
**工時估計**: 2 小時

**待實現**:
- [ ] 啟動歡迎畫面顯示最近檔案
- [ ] Pin/Unpin 常用檔案
- [ ] 清除歷史記錄

---

### 14. Line Operations 選單
**底層**: ✅ Line operations commands 已實現
**目前**: ⚠️ 只有 Keyboard shortcuts
**工時估計**: 2 小時

**待實現**:
- [ ] Menu: Edit → Line Operations
- [ ] Duplicate Line
- [ ] Move Line Up/Down
- [ ] Delete Line
- [ ] Join Lines
- [ ] Split Line
- [ ] Trim Trailing/Leading Space

---

## 📊 工時總估

| 優先級 | 功能數 | 估計工時 | 累計 |
|--------|--------|---------|------|
| 🔴 高 | 7 | 28-38 小時 | 28-38h |
| 🟡 中 | 5 | 18-25 小時 | 46-63h |
| 🟢 低 | 4 | 8-10 小時 | 54-73h |

**總估**: **46-65 小時**（約 6-9 天全職工作）

---

## 🎯 建議開發順序

### Phase 1: 基礎 UI（第 1-2 天）
1. MenuBar.tsx 基礎框架
2. StatusBar.tsx
3. Save/Save As 入口
4. Edit 選單基礎（Undo/Redo）

### Phase 2: 核心功能（第 3-5 天）
5. Cut/Copy/Paste（含底層實現）
6. AI Server Settings
7. SFTP Site Manager 入口

### Phase 3: 功能驗證（第 6-7 天）
8. Color Marks 驗證
9. Session Management 整合
10. Bookmark 強化

### Phase 4: 完善（第 8-9 天）
11. Print/Export PDF
12. Plugin Manager
13. 其他增強功能

---

## 📝 相關檔案

- `UI_ENHANCEMENT_PLAN.md` - 詳細 UI 增強計劃
- `FEATURES_IMPLEMENTED.md` - 已實現功能列表
- `FEATURE_LIST.md` - 功能統計

---

**最後更新**: 2026-03-28
**下次審閱**: 每週更新進度
