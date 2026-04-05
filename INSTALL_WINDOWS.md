# SAK Editor - Windows 安裝指南

## 系統需求

- Windows 10/11
- [Node.js 18+](https://nodejs.org/) (LTS 版本推薦)
- [Rust](https://rustup.rs/) (透過 rustup 安裝)
- [Git](https://git-scm.com/download/win)

## 安裝步驟

### 1. 安裝必要工具

**Node.js**
```powershell
# 下載並安裝: https://nodejs.org/en/download/
# 驗證安裝:
npm --version
```

**Rust (透過 rustup)**
```powershell
# 在 PowerShell 執行:
irm https://rustup.rs/install.ps1 | iex
# 或下載安裝程式: https://rustup.rs/

# 驗證安裝:
rustc --version
cargo --version
```

**WebView2 Runtime** (Windows 10 可能需要)
- 下載: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
- 選擇 "Evergreen Standalone Installer"

### 2. Clone 專案

```powershell
git clone https://github.com/bluemot/sak-note.git sak-editor
cd sak-editor
```

### 3. 安裝前端相依

```powershell
cd src-frontend
npm install
```

### 4. 安裝 Tauri CLI

```powershell
cargo install tauri-cli
```

## 開發模式執行

### 方法 1: 分開執行（推薦用於開發）

**Terminal 1 - 前端 Dev Server:**
```powershell
cd src-frontend
npm run dev
```

**Terminal 2 - Tauri App:**
```powershell
cd src-tauri
cargo tauri dev
```

### 方法 2: 一鍵執行

```powershell
cd src-frontend
npm run tauri dev
```

## Build Release 版本

```powershell
cd src-tauri
cargo tauri build
```

輸出位置:
- `src-tauri/target/release/bundle/msi/` - Windows Installer (.msi)
- `src-tauri/target/release/bundle/nsis/` - Windows Installer (.exe)

## 常見問題

### 1. "cannot find WebView2Loader.dll"
**解決**: 安裝 WebView2 Runtime (見步驟 1)

### 2. "npm install" 失敗
**解決**: 
```powershell
# 清除 cache 重試
npm cache clean --force
npm install
```

### 3. Rust build 很慢
**解決**: 第一次 build 會很慢，之後會有 cache

### 4. 缺少 Visual Studio Build Tools
**解決**: 
```powershell
# 下載 Visual Studio Build Tools
# https://visualstudio.microsoft.com/visual-cpp-build-tools/
# 安裝時選擇 "Desktop development with C++"
```

## 測試

```powershell
cd src-frontend

# 單元測試
npm test

# E2E 測試 (需要先執行 npm run dev)
npm run test:e2e
```

## 專案結構

```
sak-editor/
├── src-frontend/          # React + TypeScript 前端
│   ├── src/
│   │   ├── components/    # React 組件
│   │   ├── ui-system/     # UI Registry 系統
│   │   ├── modules/       # 功能模組
│   │   └── store/         # Zustand state
│   ├── e2e/               # Playwright E2E 測試
│   └── package.json
├── src-tauri/             # Rust + Tauri 後端
│   ├── src/
│   │   ├── vfs/           # 虛擬檔案系統
│   │   ├── modules/       # 後端模組
│   │   └── main.rs
│   └── Cargo.toml
└── tests/                 # 其他測試腳本
```

## 功能特色

- ✅ 大檔案處理 (memory-mapped)
- ✅ Markdown Editor (Rich Text)
- ✅ Hex Viewer
- ✅ SFTP 遠端檔案
- ✅ LLM 整合 (Ollama)
- ✅ WASM Plugin 系統
- ✅ 分頁編輯
- ✅ 書籤和標記

## 技術堆疊

| 層級 | 技術 |
|------|------|
| Frontend | React 18 + TypeScript + Vite |
| UI Framework | Catppuccin Theme |
| Editor | Monaco Editor + MDXEditor |
| Backend | Tauri 2.x (Rust) |
| State | Zustand |
| Testing | Playwright + Vitest |

## 支援平台

- ✅ Windows 10/11
- ✅ macOS
- ✅ Linux

---

有問題請在 GitHub Issues 回報: https://github.com/bluemot/sak-note/issues
