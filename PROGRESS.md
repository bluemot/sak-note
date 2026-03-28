# SAK Editor - Project Progress

## Documentation Index

| File | Purpose | Status |
|------|---------|--------|
| **PROGRESS.md** | This file - Project progress log and development history | ✅ Active |
| **README.md** | Project overview, quick start, features | ✅ Complete |
| **TODO.md** | TODO list with 14 items and hour estimates (54-73h) | ✅ Created 2026-03-28 |
| **FEATURES_IMPLEMENTED.md** | Complete list of implemented features (60+) | ✅ Created 2026-03-28 |
| **FEATURE_LIST.md** | Feature statistics summary | ✅ Created 2026-03-28 |
| **UI_MODULE_MAPPING.md** | UI to Module mapping - backend vs frontend gap analysis | ✅ Created 2026-03-28 |
| **UI_REGISTRY_PLAN.md** | UI Registry Pattern implementation plan with technical gotchas | ✅ Created 2026-03-28 |
| **SUBAGENT_GUIDE.md** | OpenClaw subagent usage guide (runtime:"subagent") | ✅ Created 2026-03-28 |

## Project Overview
A modern cross-platform text editor with:
- Large file handling (memory-mapped, chunked)
- LLM integration for summary and chat (Ollama cloud API)
- Color highlighting with persistence
- Hex editor mode
- Search history persistence
- SSH/SFTP remote file access via VFS
- Built-in modular architecture

## Tech Stack
- **Frontend**: React + TypeScript + Vite + Monaco Editor
- **Backend**: Tauri 2.x (Rust) for native file I/O
- **LLM**: Ollama cloud API (kimi-k2.5:cloud model)
- **VFS**: Unified virtual file system for local/SFTP

## Progress Log

### 2026-03-28 - Phase 4: Advanced UI Features

#### 1. Resizable Panels
- [x] **ResizableContainer Component** (`ui-system/components/ResizableContainer.tsx`)
  - Horizontal/vertical resize support
  - Min/max size constraints (200-400px for sidebar)
  - localStorage persistence (`sidebar-width` key)
  - Catppuccin Mocha themed handle with visual feedback
  - Mouse event handling (mousedown/mousemove/mouseup)
  - Window resize bounds checking

#### 2. Notification/Toast System Enhanced
- [x] **NotificationContainer Component** (`ui-system/components/NotificationContainer.tsx`)
  - Four notification types: success, error, warning, info
  - Auto-dismiss with configurable duration
  - Click-to-close functionality
  - Action buttons support (Undo, Retry, Dismiss)
  - Position variants: top-right (default), top-left, bottom-right, bottom-left, top-center, bottom-center
  - Updated uiStore.ts with NotificationAction interface
  - notify utility object for easy notification creation

#### 3. Plugin Dynamic Loading Foundation
- [x] **pluginLoader.ts** (`ui-system/pluginLoader.ts`)
  - Load plugins from JSON manifest (plugin.json)
  - WASM module loading with import/export handling
  - UI component registration via uiRegistry
  - Host API functions: log, error, invoke, emit, getState, alloc, free
  - Plugin lifecycle: loadPlugin, unloadPlugin, loadAllPlugins
  - Hook system: triggerHook for file events

#### 4. Keyboard Shortcuts Enhanced
- [x] **ShortcutManager** (`ui-system/ShortcutManager.ts`)
  - Global keyboard shortcut registration
  - Conflict detection between shortcuts
  - Shortcut hint display via notifications
  - PreventDefault handling for shortcuts
  - Integration with ModuleUIRegistry
  - Input field protection (ignores shortcuts in inputs)
  - Support for modifier combinations (Ctrl, Shift, Alt, Meta)

#### 5. Search Panel Enhanced
- [x] **SearchPanel Component** (`ui-system/components/SearchPanel.tsx`)
  - Collapsible/expandable bottom panel
  - Result highlighting with mark tags
  - Click-to-navigate functionality
  - Previous/Next navigation buttons
  - Result counter (X / Y)
  - Filter input for result refinement
  - Keyboard shortcuts: Escape (close), F3 (next), Shift+F3 (prev)
  - File location display with line/column numbers

#### Integration in App.tsx
- [x] ResizableContainer wraps DynamicSidebar
- [x] NotificationContainer added at top-right position
- [x] ShortcutManager initialization with custom shortcuts
- [x] Plugin loader initialization with hooks
- [x] SearchPanel conditional rendering

### 2026-03-25 - 完整自動化測試套件 (117 tests passing)

#### Tauri v2 API 兼容性修復
- [x] 更新 `@tauri-apps/api` 從 v1.5 到 v2.0
- [x] 替換 `@tauri-apps/api/tauri` 為 `@tauri-apps/api/core`
- [x] 替換 `@tauri-apps/api/dialog` 為 `@tauri-apps/plugin-dialog`
- [x] 更新所有組件 imports (App.tsx, Editor.tsx, HexViewer.tsx, LlmChat.tsx, MarkPanel.tsx)
- [x] 修復 LlmChat.tsx scrollIntoView null 檢查

#### Layer 1: Rust 單元測試 (12 tests)
- [x] VFS EditJournal 測試 - undo/redo, piece table, effective size
- [x] 所有 Rust 測試通過 `cargo test`

#### Layer 2: React 組件測試 (15 tests)
- [x] Vitest + Testing Library 設置
- [x] Toolbar 組件測試 (9 tests)
- [x] Sidebar 組件測試 (6 tests)
- [x] 所有組件測試通過 `npm run test`

#### Layer 3: Playwright E2E UI 自動化測試 (9 tests)
- [x] 真正啟動瀏覽器並點擊 UI 元素
- [x] should launch app and show welcome screen
- [x] should display all feature items
- [x] should have sidebar with three tabs
- [x] should show "No file open" message initially
- [x] should switch between sidebar tabs
- [x] should have Open File button that can be clicked
- [x] should have search functionality UI
- [x] should have toolbar with all buttons
- [x] should handle window resize
- [x] 全部 9 個測試通過 (9.5s)

#### Layer 3: Shell E2E 測試 (81 tests)
- [x] VFS Core 單元測試 (16 tests)
- [x] App Structure 測試 (34 tests)
- [x] File Operations 測試 (31 tests)

#### 一键脚本
- [x] `install.sh` / `install.bat` - 一键环境安装
- [x] `build.sh` / `build.bat` - 一键构建 release
- [x] `help.sh` - 显示可用命令
- [x] 所有脚本支持 `--help` / `help` 参数

**总计: 117 个测试全部通过**
- Rust: 12
- React: 15
- Playwright: 9
- Shell: 81

### 2026-03-23 - VFS Integration & Module System

#### VFS Architecture (Unified File Access)
- [x] **VFS Core** (`src-tauri/src/vfs/`)
  - [x] `VfsBackend` trait - unified interface for local/SFTP
  - [x] `VfsFile` trait - file operations (read/write/seek)
  - [x] `EditJournal` - undo/redo with piece table for correct logical→physical mapping
  - [x] `VfsManager` - global handle cache, routes to appropriate backend

- [x] **LocalBackend** - mmap-based local file access
  - [x] Mmap caching for performance
  - [x] `read_dir()`, `metadata()`, `open_read()`, `open_write()`
  - [x] Directory listing support

- [x] **SftpBackend** - SSH/SFTP remote file access
  - [x] Connection management (password/key auth)
  - [x] Remote file read/write via SFTP
  - [x] Directory listing via SFTP

- [x] **Optimizations Applied** (via Gemini CLI review)
  - [x] Fixed Clone bug - `VfsFileHandle` now shares `EditJournal` via `Arc<RwLock>`
  - [x] Piece table implementation - correct logical_to_physical mapping
  - [x] Mmap caching - file handles reused, not recreated on each read
  - [x] Removed duplicate state tracking

#### Modular System
- [x] **Module Registry** - JSON-RPC style module execution
- [x] **FileModule** - large file handling via VfsManager
  - `open`, `close`, `read`, `read_text`
  - `insert`, `delete`, `replace`
  - `save`, `undo`, `redo`
  - `get_info`, `get_hex`
  - `read_dir`, `stat`
- [x] **SftpModule** - SSH/SFTP operations (integrating with VfsManager)
  - `connect`, `disconnect`
  - `open`, `close`, `read`, `read_text`, `write`
  - `list_dir`, `stat`, `mkdir`, `rmdir`, `unlink`
- [x] **LlmModule** - Ollama cloud API integration
  - `chat` - send messages, receive responses
  - `list_models` - discover available models
  - `get_context`, `clear_context` - conversation history
  - `set_system_prompt` - customize AI behavior
  - `summarize`, `ask_about_file` - specialized queries
- [x] **MarksModule** - color highlighting system
  - `create`, `update`, `delete` marks
  - `get`, `get_at` - retrieve marks
  - `clear`, `export`, `import`

#### Frontend Components
- [x] **App.tsx** - main layout with sidebar
- [x] **Sidebar.tsx** - tabs: Info / Chat / Marks
- [x] **LlmChat.tsx** - chat interface with Ollama
  - Catppuccin Mocha dark theme
  - Message history with timestamps
  - Model selector dropdown
  - Connection status indicator
- [x] **Editor.tsx** - Monaco-based text editor
  - Chunked loading for large files
  - Virtual scrolling placeholder
- [x] **HexViewer.tsx** - hex view with ASCII panel
- [x] **Toolbar.tsx** - search, view toggle, file operations

### 2026-03-22 - UI Integration & LLM
- [x] Integrated LlmChat into sidebar with Catppuccin theme
- [x] Ollama cloud API integration (kimi-k2.5:cloud)
- [x] React frontend build fixes
- [x] TypeScript unused variable cleanup

### 2026-03-21 - Project Initialization & Core Features
- [x] Created project directory structure
- [x] Initialized npm project with Vite
- [x] Set up Tauri 2.x configuration
- [x] Created core file engine (Rust) with memory-mapped file support
  - [x] Chunk manager with 64KB chunks for large files
  - [x] Memory-mapped file I/O using memmap2 crate
  - [x] Hex viewer utilities
  - [x] Global file cache with DashMap

### 2026-03-21 - Edit, Search & Save Features
- [x] **EditableFileManager** - Full read/write support
  - [x] Insert/Delete/Replace operations with EditOp enum
  - [x] Undo/Redo history (journal-based)
  - [x] Modified regions tracking
  - [x] Effective size calculation
- [x] **Search Engine** - Boyer-Moore-Horspool algorithm
- [x] **Save functionality** - atomic replace via temp file
- [x] **Tauri Commands** for all operations

## Architecture

```
sak-editor/
├── src-tauri/src/
│   ├── lib.rs                 # Tauri app entry
│   ├── main.rs                # Binary entry
│   ├── modular/               # Module registry system
│   │   └── mod.rs
│   ├── modules/                # Feature modules
│   │   ├── file_module.rs     # File ops via VFS
│   │   ├── sftp_module.rs      # SFTP via VFS
│   │   ├── llm_module.rs       # Ollama integration
│   │   └── marks_module.rs     # Color marks
│   ├── vfs/                    # Virtual File System
│   │   ├── mod.rs             # Traits + EditJournal
│   │   ├── manager.rs         # VfsManager (global cache)
│   │   ├── local.rs           # LocalBackend (mmap)
│   │   └── remote.rs          # SftpBackend (ssh2)
│   └── file_engine/            # Legacy chunk system
│       └── chunk.rs           # (Still used by some code)
├── src-frontend/src/
│   ├── App.tsx
│   ├── components/
│   │   ├── Sidebar.tsx
│   │   ├── LlmChat.tsx
│   │   ├── Editor.tsx
│   │   ├── HexViewer.tsx
│   │   └── Toolbar.tsx
│   └── index.css
├── tests/
│   ├── e2e/
│   │   └── playwright/
│   │       └── ui-automation.spec.ts  # 9 tests
│   ├── unit/
│   │   └── test_vfs_core.sh         # 16 tests
│   └── run_all_tests.sh             # test runner
├── install.sh / install.bat   # One-click setup
├── build.sh / build.bat       # One-click build
├── PROGRESS.md
└── README.md
```

## Module Capabilities

### file module (v2.1.0)
| Capability | Description |
|------------|-------------|
| `open` | Open file via VfsManager |
| `close` | Close file handle |
| `read` | Read raw bytes |
| `read_text` | Read as UTF-8 text |
| `insert` | Insert bytes at offset |
| `delete` | Delete bytes at offset |
| `replace` | Replace bytes at offset |
| `save` | Save changes |
| `undo` | Undo last operation |
| `redo` | Redo last undone |
| `get_info` | Get file metadata |
| `get_hex` | Get hex view rows |
| `read_dir` | List directory contents |
| `stat` | Get file/directory info |

### sftp module (v2.0.0)
| Capability | Description |
|------------|-------------|
| `connect` | Connect to SSH server |
| `disconnect` | Close SSH connection |
| `open` | Open remote file |
| `close` | Close remote file |
| `read` | Read remote bytes |
| `read_text` | Read remote as text |
| `write` | Write to remote |
| `list_dir` | List remote directory |
| `stat` | Remote file info |
| `mkdir` | Create remote directory |
| `rmdir` | Remove remote directory |
| `unlink` | Delete remote file |

### llm module (v1.0.0)
| Capability | Description |
|------------|-------------|
| `chat` | Send message, receive AI response |
| `list_models` | List available Ollama models |
| `get_context` | Get conversation history |
| `clear_context` | Clear conversation history |
| `set_system_prompt` | Set system prompt |
| `summarize` | Summarize current file |
| `ask_about_file` | Ask AI about file content |
| `generate` | Generate text with AI |

## Testing Strategy (3-Layer)

### Layer 1: Rust Unit Tests (12 tests)
Run: `cargo test --lib vfs::`
- EditJournal operations
- Piece table functionality
- Undo/Redo system

### Layer 2: React Component Tests (15 tests)
Run: `cd src-frontend && npm test`
- Toolbar component (9 tests)
- Sidebar component (6 tests)
- Vitest + React Testing Library

### Layer 3: E2E Tests

#### Playwright UI Automation (9 tests)
Run: `npm run test:e2e`
- Real browser automation
- Click UI elements
- Verify visual state
- All tests pass in ~9.5s

#### Shell Script Tests (81 tests)
Run: `./tests/run_all_tests.sh`
- VFS Core: 16 tests
- App Structure: 34 tests
- File Operations: 31 tests

**Total: 117 tests passing**

## Quick Commands

```bash
# Setup
./install.sh help      # Show install help
./install.sh          # Install all dependencies

# Development
npm run dev           # Start dev server
npm run tauri-dev     # Start Tauri dev mode

# Build
./build.sh help       # Show build help
./build.sh            # Build release version

# Test
npm test              # Run all tests (117 total)
npm run test:e2e      # Run Playwright UI tests
```

## Next Steps
1. [x] ~~Test frontend on local machine with GUI~~ ✅ Ubuntu desktop 可執行
2. [x] ~~Add comprehensive test suite~~ ✅ 117 tests passing
3. [ ] Implement SFTP backend → VfsManager integration
4. [ ] Add file browser UI component
5. [ ] Implement virtual scrolling in Editor.tsx
6. [ ] Add marks persistence (marks_module → file_engine integration)

## Known Issues
- [x] ~~Headless server can't run Tauri GUI~~ ✅ Ubuntu desktop 可正常執行
- [x] ~~Tauri API v1/v2 mismatch~~ ✅ 已修復
- [ ] SFTP backend exists but not fully wired to VfsManager
- [ ] file_engine chunk system somewhat redundant with VFS
- [ ] cargo build 產生 116 warnings 待整理

## Git Commits
- `a7116c5` - Fix Tauri v2 API compatibility and add Playwright tests (117 tests)
- `72e1520` - Add comprehensive test suite and install/build scripts
- `c555d12` - Add read_dir and stat to file module, integrate with VfsManager
- `b896245` - VFS optimization: fix Clone bug, piece table, Mmap cache
- `9f16421` - Fix TypeScript unused variable warnings
- `b562b2c` - Add React LLM Chat UI component
- `0e238d4` - LLM module: Ollama API integration with cloud support

## Notes
- VFS is the unifying layer - once SftpBackend is wired, all modules support remote files
- Focus on large file performance from day one
- Use streaming for file operations
- Keep UI responsive with async operations
- Testing follows 3-layer strategy: Rust unit → React component → E2E automation

---

### 2026-03-27 - WASM Plugin System + Debug Logging

#### WASM Plugin System Complete
- [x] **Plugin Runtime**: wasmtime 23 + WASI Preview 1
  - [x] `wasm_engine.rs` - WASM execution with fuel/memory limits
  - [x] `manager.rs` - Plugin discovery, loading, lifecycle
  - [x] `bridge.rs` - Safe API bridge between plugins and editor
  - [x] `commands.rs` - 11 Tauri commands for plugin management
  
- [x] **Security Model** (3-layer protection)
  - [x] Fuel mechanism - prevent infinite loops (10M instructions limit)
  - [x] Memory cap - prevent OOM (64MB hard limit)
  - [x] VFS permission checks - unified file access control
  
- [x] **Sample Plugin** (`plugins/sample-plugin/`)
  - [x] uppercase, word-count, sort-lines commands
  - [x] plugin.json manifest with UI declarations
  - [x] Cross-platform .wasm binary

- [x] **Host API for Plugins**
  - [x] `sak_log()` - Logging
  - [x] `sak_get_content()` / `sak_set_content()` - File operations
  - [x] `sak_show_notification()` - UI notifications
  - [x] `sak_get_selection()` - Editor selection

#### Notepad++ Core Features Complete
- [x] **Session Management**: Save/restore editor sessions
- [x] **Print Manager**: Cross-platform printing (Linux/Win/Mac)
- [x] **Find in Files**: Cross-file search with regex
- [x] **Go to Line**: Ctrl+G dialog
- [x] **Recent Files**: Persistent list with timestamps

#### LLM Integration
- [x] **MCP Server**: 30+ tools for AI assistants
- [x] **LLM Guide**: `docs/LLM_GUIDE.md` for easy integration
- [x] **Semantic Analysis**: Code parsing, natural language queries

#### Testing & Debugging
- [x] **GUI Tests**: 46 Playwright E2E tests
- [x] **Debug Logging**: Complete `debug-logging` branch
  - [x] `scripts/run-with-logs.sh` - Automatic logging
  - [x] `scripts/view-logs.sh` - Log viewer
  - [x] Full Open File workflow tracing (29 logs in App.tsx, 17 in Editor.tsx, 40 in backend)
- [x] **Dialog Fix**: Fixed `tauri.conf.json` plugins configuration

#### Git Branches
- [x] `main` - Production code (all features)
- [x] `debug-logging` - Debug version with full tracing

**Total**: 117 Rust tests + 46 Playwright tests passing, 0 build errors

#### In Progress / Next
- [ ] GUI testing in actual user environment
- [ ] Verify Open File workflow with debug logs
- [ ] Plugin marketplace (future)
