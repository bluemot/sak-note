# SAK Editor - 完整功能列表 (2026-03-28)

## 已實現功能總覽

### 📁 檔案操作 (File Operations)
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| Open File (本機) | ✅ | ✅ auto-test.sh | dialog 已修復 |
| Open File (SFTP) | ✅ | ⚠️ 需手動測試 | VFS 支援 |
| Close File | ✅ | ✅ | |
| Save File | ✅ | ⚠️ UI 待驗證 | Rust command 已實現 |
| Save As | ✅ | ⚠️ UI 待驗證 | Rust command 已實現 |
| Recent Files | ✅ | ✅ | 持續化儲存 |
| Session Save/Load | ✅ | ⚠️ | 工作階段管理 |

### 📝 編輯功能 (Editing)
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| Text Editor (Monaco) | ✅ | ✅ | Chunked loading |
| Hex Viewer | ✅ | ✅ | |
| Insert/Delete/Replace | ✅ | ✅ Rust tests | 位元組層級 |
| Undo/Redo | ✅ | ✅ Rust tests | EditJournal |
| Go to Line | ✅ | ✅ | Ctrl+G dialog |
| Line Operations | ✅ | ✅ | 上移/下移/複製/刪除 |

### 🔍 搜尋與替換 (Search)
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| Find in Current File | ✅ | ✅ | Boyer-Moore-Horspool |
| Replace in Current File | ✅ | ⚠️ | |
| Find in Files | ✅ | ✅ | 多檔案搜尋 |
| Search History | ✅ | ✅ | 持續化 |

### 🎨 標記與書籤 (Marks & Bookmarks)
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| Color Marks | ✅ | ⚠️ UI 待驗證 | Sidebar 已連結 |
| Create/Update/Delete Mark | ✅ | ✅ Rust tests | |
| Bookmarks | ✅ | ✅ | BookmarkPanel.tsx |
| Export/Import Marks | ✅ | ⚠️ | Rust module 已實現 |

### 🌐 SFTP/SSH 遠端檔案
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| SFTP Site Manager | ✅ | ⚠️ UI 待驗證 | SftpSiteManager.tsx |
| SFTP Connect | ✅ | ⚠️ | VFS Backend |
| Remote File Read | ✅ | ✅ VFS tests | |
| Remote File Write | ✅ | ✅ VFS tests | |
| Directory Listing | ✅ | ✅ VFS tests | |

### 🤖 LLM 整合
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| Chat Panel | ✅ | ✅ | LlmChat.tsx |
| Ollama Integration | ✅ | ⚠️ | kimi-k2.5:cloud |
| Model Selection | ✅ | ✅ | |
| File Summary | ✅ | ⚠️ | |
| AI Server Settings | ⚠️ | ❌ | **UI 未完成** |

### 🔌 WASM Plugin System
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| Plugin Runtime | ✅ | ✅ Rust tests | wasmtime 23 |
| WASI Preview 1 | ✅ | ✅ | |
| Security Model | ✅ | ✅ | Fuel limits, VFS permissions |
| Sample Plugin | ✅ | ✅ | plugins/sample-plugin/ |
| Plugin Discovery | ✅ | ✅ | |
| Plugin Load/Unload | ✅ | ✅ | |
| Event Broadcasting | ✅ | ✅ | |

### 📊 MCP Server (AI Assistant Tools)
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| MCP Server | ✅ | ✅ | 30+ tools |
| LLM Guide | ✅ | ✅ | docs/LLM_GUIDE.md |
| Semantic Analysis | ✅ | ✅ | Code parsing |

### 🖨️ 列印與匯出
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| Print Manager | ✅ | ⚠️ | Cross-platform |
| Export PDF | ✅ | ⚠️ | |

### 🎨 UI/UX
| 功能 | 狀態 | 測試 | 備註 |
|------|------|------|------|
| Toolbar | ✅ | ✅ | Save 按鈕待加 |
| Sidebar | ✅ | ✅ | Info/Chat/Marks tabs |
| Tabs | ✅ | ✅ | Tabs.tsx |
| Dark Theme | ✅ | ✅ | Catppuccin Mocha |
| Find/Replace Dialog | ✅ | ✅ | |
| Go to Line Dialog | ✅ | ✅ | |
| Session Manager Dialog | ✅ | ✅ | |

---

## 自動化測試覆蓋率

| 類型 | 測試數 | 通過 | 失敗 | 跳過 |
|------|--------|------|------|------|
| Rust Unit Tests | 12 | 12 | 0 | 0 |
| React Component Tests | 15 | 15 | 0 | 0 |
| Playwright E2E Tests | 75 | 60 | 0 | 15 |
| Shell Script Tests | 81 | 81 | 0 | 0 |
| **總計** | **183** | **168** | **0** | **15** |

**15 個跳過的測試**：Plugin System 測試（需要 Tauri runtime，純瀏覽器無法執行）

---

## 🔧 需要修復/補充的功能

### 高優先級 (Tom 報告)
1. **Mark Panel UI** - Sidebar 已連結 MarkPanel，需驗證功能
2. **SFTP Site Manager UI** - 確認開啟方式
3. **AI Server Settings UI** - 需新增設定介面
4. **Save Button** - Toolbar 需新增 Save 按鈕

### 中優先級
5. **Plugin System E2E Tests** - 需要 Tauri 環境測試
6. **Find in Files Full Test** - 完整流程測試
7. **Print Functionality** - 實際列印測試

---

## 技術棧

- **Frontend**: React 18 + TypeScript + Vite + Monaco Editor
- **Backend**: Tauri 2.x (Rust)
- **VFS**: 統一虛擬檔案系統 (Local/SFTP)
- **Plugin**: wasmtime 23 + WASI Preview 1
- **LLM**: Ollama cloud API (kimi-k2.5:cloud)
- **Testing**: Vitest + Playwright + Shell Scripts

## 檔案結構
```
sak-editor/
├── src-frontend/src/
│   ├── App.tsx                 # 主應用
│   ├── components/
│   │   ├── Toolbar.tsx         # 工具列 (Open/Save/Close)
│   │   ├── Sidebar.tsx         # 側邊欄 (Info/Chat/Marks)
│   │   ├── Editor.tsx          # Monaco 編輯器
│   │   ├── HexViewer.tsx       # 十六進制檢視
│   │   ├── LlmChat.tsx         # AI 對話
│   │   ├── MarkPanel.tsx       # 顏色標記
│   │   ├── BookmarkPanel.tsx   # 書籤
│   │   ├── SftpSiteManager.tsx # SFTP 站台管理
│   │   ├── FindReplace.tsx     # 搜尋替換
│   │   ├── GoToLineDialog.tsx  # 跳至行
│   │   ├── SessionManager.tsx  # 工作階段
│   │   └── RecentFiles.tsx     # 最近檔案
│   └── services/
│       └── pluginService.ts    # Plugin 初始化
├── src-tauri/src/
│   ├── lib.rs                  # Tauri 主入口
│   ├── vfs/                    # 虛擬檔案系統
│   ├── modules/                # 功能模組
│   ├── file_engine/            # 檔案引擎
│   └── plugin_runtime/         # WASM Plugin
└── scripts/
    ├── auto-test.sh            # UI 自動化測試
    ├── install.sh              # 環境安裝
    └── build.sh                # 建置腳本
```
