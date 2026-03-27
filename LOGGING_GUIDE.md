# SAK Editor 日誌系統使用指南

## 日誌位置

### 後端日誌 (Rust)
```
~/.config/sak-editor/logs/
├── sak-editor.log          # 主要日誌
├── sak-editor-error.log    # 錯誤日誌
└── plugin/                 # 插件日誌
    └── {plugin-id}.log
```

### 前端日誌 (JavaScript)
```
~/.config/sak-editor/logs/
└── frontend.log            # 前端操作日誌
```

## 查看日誌

### 方法 1: 使用腳本
```bash
# 查看所有日誌
./scripts/view-logs.sh

# 只看錯誤
./scripts/view-logs.sh --errors

# 只看插件
./scripts/view-logs.sh --plugin

# 即時追蹤
./scripts/view-logs.sh --follow
```

### 方法 2: 手動查看
```bash
# 後端日誌
tail -f ~/.config/sak-editor/logs/sak-editor.log

# 前端日誌
tail -f ~/.config/sak-editor/logs/frontend.log

# 搜尋特定問題
grep "dialog" ~/.config/sak-editor/logs/sak-editor.log
```

## 日誌級別

- **ERROR**: 錯誤，需要修復
- **WARN**: 警告，可能影響功能
- **INFO**: 一般信息
- **DEBUG**: 詳細調試信息
- **TRACE**: 最詳細追蹤

## 日誌內容範例

```
[2026-03-27 13:25:30 INFO] SAK Editor v0.1.0 started
[2026-03-27 13:25:30 INFO] Plugin system initialized
[2026-03-27 13:25:35 INFO] User clicked: Open button
[2026-03-27 13:25:35 DEBUG] Calling dialog plugin
[2026-03-27 13:25:35 ERROR] Dialog plugin failed: permission denied
[2026-03-27 13:25:35 INFO] Fallback to default dialog
```

## 啟用 DEBUG 級別

```bash
# 設置環境變量
export SAK_LOG_LEVEL=debug
./src-tauri/target/release/sak-editor
```

## 常見問題排查

### Dialog 無法開啟
```bash
grep -i "dialog" ~/.config/sak-editor/logs/sak-editor.log
```

### 插件載入失敗
```bash
grep -i "plugin" ~/.config/sak-editor/logs/sak-editor.log
tail ~/.config/sak-editor/logs/plugin/*.log
```

### WASM 執行錯誤
```bash
grep -i "wasm" ~/.config/sak-editor/logs/sak-editor-error.log
```
