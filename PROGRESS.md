# SAK Editor - Project Progress

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

## Next Steps
1. [x] ~~Test frontend on local machine with GUI~~ ✅ 可在 Ubuntu desktop 環境執行
2. [ ] Implement SFTP backend → VfsManager integration
3. [ ] Add file browser UI component
4. [ ] Implement virtual scrolling in Editor.tsx
5. [ ] Add marks persistence (marks_module → file_engine integration)

## Known Issues
- [x] ~~Headless server can't run Tauri GUI~~ ✅ Ubuntu desktop 環境可正常執行（Rust 1.94.0）
- [ ] SFTP backend exists but not fully wired to VfsManager
- [ ] file_engine chunk system somewhat redundant with VFS
- [ ] cargo build 產生 116 warnings有待整理

## Git Commits
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

## 2026-03-25 - Tauri Dev 環境修復
- 補充安裝系統依賴：`libgtk-3-dev`, `libjavascriptcoregtk-4.1-dev`, `libsoup-3.0-dev`
- 修復 `tauri.conf.json`：加入 `app.frontend.devUrl: "http://localhost:5173"` 設定
- 前端 Vite dev server 正常運行在 `http://localhost:5173`
- Tauri dev 可正確載入前端頁面

## 2026-03-24 - 成功在 Ubuntu Desktop 執行 GUI
- 安裝 Rust toolchain (1.94.0)
- `cargo build --release` 編譯成功（116 warnings）
- Tauri app 可正常啟動，視窗、選單、搜尋列、視圖切換皆正常
- 前端 Vite dev server 可單獨運行（無 backend）
- 截圖已保存至 `/home/ubuntu/.openclaw/media/browser/`
