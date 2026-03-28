# SAK Editor - UI 功能 vs Module 對照表

**說明**: 本表列出每個 UI 功能對應的底層 Module，以及目前的連結狀態

---

## 📁 File Menu 功能對應

| UI 功能 | Module | Command | UI 狀態 | 備註 |
|---------|--------|---------|---------|------|
| **Open** (本機) | file_module | `file.open` | ✅ 已連結 | dialog 已修復 |
| **Open Remote** (SFTP) | sftp_module | `sftp.connect` + `sftp.open` | ❌ 未連結 | 需 Menu + Toolbar 入口 |
| **Save** | file_module | `file.save` | ❌ 未連結 | Toolbar 缺 Save 按鈕 |
| **Save As** | file_module | `file.save_as` | ❌ 未連結 | Menu 缺 File 選單 |
| **Recent Files** | file_module | `file.open` (記錄在 Rust) | ⚠️ 部分連結 | 有組件但沒整合到 Menu |
| **Save Session** | (內置) | session 相關 | ⚠️ 部分連結 | SessionManager.tsx 存在 |
| **Load Session** | (內置) | session 相關 | ⚠️ 部分連結 | 需 Menu 入口 |
| **Print** | print_manager | `print` / `export_pdf` | ❌ 未連結 | 需 Menu + Toolbar 入口 |
| **Export PDF** | print_manager | `export_pdf` | ❌ 未連結 | 需 Menu 入口 |
| **Exit** | (系統) | - | ❌ 未連結 | Menu 不存在 |

**Module 狀態**: FileModule ✅ 完整，但 UI 入口缺失嚴重

---

## 📝 Edit Menu 功能對應

| UI 功能 | Module | Command | UI 狀態 | 備註 |
|---------|--------|---------|---------|------|
| **Undo** | file_module | `file.undo` | ⚠️ 待驗證 | 需 Edit 選單 |
| **Redo** | file_module | `file.redo` | ⚠️ 待驗證 | 需 Edit 選單 |
| **Cut** | ❌ 未實現 | - | ❌ 未實現 | Module 和 UI 都缺 |
| **Copy** | ❌ 未實現 | - | ❌ 未實現 | Module 和 UI 都缺 |
| **Paste** | ❌ 未實現 | - | ❌ 未實現 | Module 和 UI 都缺 |
| **Find** | file_module | `file.search` | ⚠️ 部分連結 | Toolbar 有搜尋但功能簡單 |
| **Replace** | file_module | `file.replace` | ⚠️ 待驗證 | FindReplace.tsx 存在但連結待確認 |
| **Find in Files** | file_module | `file.search` (多檔案) | ⚠️ 部分連結 | FindInFiles.tsx 存在 |
| **Go to Line** | file_module | `ui_goto_line` | ✅ 已連結 | Ctrl+G 有對話框 |
| **Bookmarks** | marks_module | Bookmark 相關 | ⚠️ 部分連結 | BookmarkPanel.tsx 存在 |
| **Marks** | marks_module | `marks.create` 等 | ⚠️ 部分連結 | MarkPanel.tsx 存在，連結待確認 |
| **Line Operations** | file_module | Line 操作 commands | ⚠️ 待驗證 | 有底層命令但沒 UI |

**Module 狀態**: 
- file_module ✅ 完整 (但缺 Cut/Copy/Paste)
- marks_module ✅ 完整
- **缺**: Clipboard module (Cut/Copy/Paste)

---

## 👁️ View Menu 功能對應

| UI 功能 | Module | Command | UI 狀態 | 備註 |
|---------|--------|---------|---------|------|
| **Text View** | file_module | `file.read_text` | ✅ 已連結 | Toolbar 有切換 |
| **Hex View** | file_module | `file.get_hex` | ✅ 已連結 | HexViewer.tsx 完整 |
| **Sidebar** | (UI) | - | ✅ 已連結 | Sidebar.tsx 完整 |
| **Toolbar** | (UI) | - | ✅ 已連結 | Toolbar.tsx 完整 |
| **Status Bar** | ❌ 未實現 | - | ❌ 未實現 | 需新增組件 |
| **Zoom In/Out** | ❌ 未實現 | - | ❌ 未實現 | Monaco Editor 支援但沒 UI |
| **Full Screen** | (系統) | - | ❌ 未連結 | 需 Menu 入口 |

**Module 狀態**: 純 UI 功能，View Module 不需要

---

## 🔍 Search Panel 功能對應 (⭐新增)

| UI 功能 | Module | Command | UI 狀態 | 備註 |
|---------|--------|---------|---------|------|
| **Find in Current File** | file_module | `file.search` | ❌ 未連結 | 需 SearchPanel.tsx |
| **結果列表顯示** | file_module | `file.search` 返回 | ❌ 未實現 | 需下方結果面板 |
| **結果高亮** | (UI) | - | ❌ 未實現 | 需 Editor 整合 |
| **上一個/下一個** | file_module | `file.search` (不同 offset) | ❌ 未連結 | 需快捷鍵 F3 |
| **搜尋歷史** | (內置) | - | ❌ 未實現 | 需持續化 |

**Module 狀態**: file_module ✅ 已支援，缺 UI 整合

---

## ⚙️ Settings Menu 功能對應

| UI 功能 | Module | Command | UI 狀態 | 備註 |
|---------|--------|---------|---------|------|
| **AI Server Settings** | llm_module | `llm.list_models` 等 | ❌ 未連結 | 完全沒有設定介面 |
| **Preferences** | (內置) | - | ❌ 未實現 | 需設定系統 |
| **Keyboard Shortcuts** | (UI) | - | ❌ 未實現 | 需快捷鍵設定介面 |

**Module 狀態**: llm_module ✅ 已連線 Ollama，但沒設定 UI

---

## 🔌 Plugins Menu 功能對應

| UI 功能 | Module | Command | UI 狀態 | 備註 |
|---------|--------|---------|---------|------|
| **Plugin Manager** | plugin_runtime | Plugin 管理 commands | ❌ 未連結 | WASM Plugin 系統完整但缺管理 UI |
| **動態外掛選單** | plugin_runtime | 載入的外掛 | ❌ 未實現 | 需根據載入外掛動態生成選單 |

**Module 狀態**: plugin_runtime ✅ WASM Plugin System 完整

---

## 🤖 Sidebar Chat 功能對應

| UI 功能 | Module | Command | UI 狀態 | 備註 |
|---------|--------|---------|---------|------|
| **Chat Interface** | llm_module | `llm.chat` | ✅ 已連結 | LlmChat.tsx 完整 |
| **Model Selection** | llm_module | `llm.list_models` | ✅ 已連結 | 下拉選單正常 |
| **File Summary** | llm_module | `llm.summarize` | ⚠️ 待驗證 | 功能存在但使用流程待確認 |
| **Ask about File** | llm_module | `llm.ask_about_file` | ⚠️ 待驗證 | 功能存在但使用流程待確認 |

**Module 狀態**: llm_module ✅ 完整連結

---

## 🎨 Sidebar Marks 功能對應

| UI 功能 | Module | Command | UI 狀態 | 備註 |
|---------|--------|---------|---------|------|
| **Mark List** | marks_module | `marks.get` | ⚠️ 待驗證 | MarkPanel.tsx 存在 |
| **Create Mark** | marks_module | `marks.create` | ⚠️ 待驗證 | 按鈕存在但連結待確認 |
| **Delete Mark** | marks_module | `marks.delete` | ⚠️ 待驗證 | 功能待確認 |
| **Editor Highlight** | marks_module | `marks.get_at` | ❌ 未連結 | Editor 未顯示標記 |
| **Jump to Mark** | marks_module | 配合 Editor | ⚠️ 待驗證 | 點擊標記跳轉功能 |

**Module 狀態**: marks_module ✅ 完整，UI 連結待驗證

---

## 🌐 SFTP Site Manager 功能對應

| UI 功能 | Module | Command | UI 狀態 | 備註 |
|---------|--------|---------|---------|------|
| **Site List** | sftp_module | `sftp.connect` + 記錄 | ⚠️ 部分連結 | SftpSiteManager.tsx 存在 |
| **Add Site** | sftp_module | `sftp.connect` | ⚠️ 部分連結 | 對話框存在但入口缺失 |
| **Edit Site** | (內置) | 站台設定 | ⚠️ 部分連結 | 功能存在但流程待確認 |
| **Connect** | sftp_module | `sftp.connect` | ⚠️ 部分連結 | 需 File → Open Remote 入口 |
| **Remote File Browser** | sftp_module | `sftp.list_dir` | ❌ 未連結 | 需遠端檔案瀏覽器 UI |

**Module 狀態**: sftp_module ✅ 完整，但缺整合入口

---

## 📊 統計總結

### Module 完成度

| Module | 狀態 | UI 連結率 | 主要問題 |
|--------|------|-----------|---------|
| file_module | ✅ 完整 | ~40% | Save/Search/Undo 缺入口 |
| marks_module | ✅ 完整 | ~30% | Editor highlight 未實現 |
| llm_module | ✅ 完整 | ~70% | AI Settings 缺設定介面 |
| sftp_module | ✅ 完整 | ~20% | 完全缺入口 |
| plugin_runtime | ✅ 完整 | ~10% | 完全缺管理 UI |
| print_manager | ✅ 完整 | ~0% | 完全缺入口 |
| clipboard | ❌ 未實現 | - | Module 和 UI 都缺 |

### UI 功能分佈

| Menu/區域 | 功能數 | 已連結 | 未連結 | 缺失 Module |
|-----------|--------|--------|--------|-------------|
| File Menu | 9 | 1 (Open) | 8 | - |
| Edit Menu | 11 | 1 (Go to Line) | 10 | clipboard |
| View Menu | 7 | 3 | 4 | status_bar |
| Search Panel | 5 | 0 | 5 | - |
| Settings Menu | 3 | 0 | 3 | - |
| Plugins Menu | 2 | 0 | 2 | - |
| Sidebar Chat | 4 | 3 | 1 | - |
| Sidebar Marks | 5 | 1 | 4 | - |
| **總計** | **46** | **10** | **36** | **clipboard, status_bar** |

---

## 🎯 優先級建議

### Phase 1: 核心功能補丁
1. **Save/Save As** (file_module 已有)
2. **Undo/Redo** (file_module 已有)
3. **Search Panel** (file_module 已有)

### Phase 2: 常用功能
4. **Cut/Copy/Paste** (需新建 clipboard module)
5. **Status Bar** (純 UI，無底層依賴)
6. **AI Settings** (llm_module 已有)

### Phase 3: 進階功能
7. **SFTP 整合** (sftp_module 已有)
8. **Marks Editor Highlight** (marks_module 已有)
9. **Plugin Manager** (plugin_runtime 已有)

---

## 📝 備註

- **✅ 已連結**: UI 和 Module 正確連結且功能可用
- **⚠️ 部分連結**: 組件存在但功能連結待驗證
- **❌ 未連結**: Module 存在但 UI 入口缺失
- **❌ 未實現**: Module 和 UI 都不存在

**最後更新**: 2026-03-28
