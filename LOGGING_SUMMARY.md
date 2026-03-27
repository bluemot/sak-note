# SAK Editor Plugin System - Logging and Testing Summary

## Summary of Changes

### 1. Added Logging to Key Components

#### Backend (Rust) Logging Added:

**plugin_runtime/commands.rs**
- Added comprehensive logging to all Tauri commands:
  - `plugin_init`: Logs initialization start/success
  - `plugin_discover`: Logs discovery process, found plugins, and errors
  - `plugin_load_all`: Logs load status for each plugin
  - `plugin_load`: Logs plugin ID, discovery path, and load result
  - `plugin_unload`: Logs unload attempts and results
  - `plugin_list_loaded`: Logs loaded plugin count and details
  - `plugin_get_info`: Logs info retrieval requests
  - `plugin_execute`: Logs capability execution with parameters
  - `plugin_set_enabled`: Logs enable/disable changes
  - `plugin_get_capabilities`: Logs capability discovery
  - `plugin_broadcast_event`: Logs event broadcasting
  - `plugin_get_directory`: Logs directory path resolution

**plugin_runtime/manager.rs**
- Added logging to plugin manager lifecycle:
  - Plugin directory creation and configuration
  - Discovery process (manifest files found, valid plugins, errors)
  - Plugin loading (success/failure with reasons)
  - Capability execution with input/output
  - Event broadcasting to plugins
  - Plugin enable/disable status changes

**plugin_runtime/wasm_engine.rs**
- Added detailed WASM execution logging:
  - WASM engine creation and configuration
  - Plugin loading (WASM file reading, compilation, WASI setup)
  - Host function registration
  - Module instantiation and memory allocation
  - Capability execution with memory operations
  - Event handling in WASM
  - Plugin shutdown

**plugin_runtime/bridge.rs**
- Added bridge operation logging:
  - Request/response handling
  - File operations (read/write)
  - Settings access
  - Notification display
  - Command execution

**plugin_runtime/mod.rs**
- Added logging for plugin system initialization

**main.rs**
- Added file-based logging initialization with timestamped log files
- Logs stored in `~/.config/sak-editor/logs/`
- Configurable via `SAK_LOG_LEVEL` environment variable

### 2. Frontend Logging Added:

**src/services/pluginService.ts (NEW)**
- Complete TypeScript service for plugin API:
  - All plugin operations with console logging
  - Initialization with detailed status
  - Discovery, loading, execution with parameter logging
  - Error handling with detailed messages

**App.tsx (UPDATED)**
- Added plugin initialization in useEffect
- Logs file open/close operations
- Logs view mode changes
- Logs state updates
- Broadcasts editor events to plugins

### 3. Playwright E2E Tests Created

**tests/e2e/playwright/plugin-system.spec.ts**
Comprehensive test suite covering:

- **Plugin Discovery Tests:**
  - Test plugin discovery returns valid manifests
  - Verify manifest structure

- **Plugin Loading Tests:**
  - Test load all plugins
  - Test load specific plugin
  - Verify load status structure

- **Plugin Execution Tests:**
  - Test capability execution
  - Verify execution results

- **Error Handling Tests:**
  - Non-existent plugin loading
  - Invalid plugin IDs
  - Capability execution on missing plugin
  - Duplicate plugin load handling

- **Plugin Management Tests:**
  - List loaded plugins
  - Get plugin info
  - Enable/disable plugins
  - Unload plugins
  - Broadcast events

All tests include comprehensive console logging that will be captured by Playwright.

### 4. Log Viewer Script Created

**scripts/view-logs.sh**
Features:
- View logs with filtering options
- Follow logs in real-time (`-f`)
- Filter by plugin (`-p`)
- Filter by log level (`-e`, `-w`, `-i`, `-d`)
- Clear logs (`--clear`)
- List log files (`--list`)

### 5. Cargo.toml Dependencies Added
```toml
env_logger = "0.11"
simplelog = "0.12"
```

### 6. Build Verification

Successfully built with:
```bash
npm run build
```

Result:
- Frontend: Built successfully (174KB JS bundle)
- Backend: Built successfully (27MB release binary)
- All Rust warnings addressed (only minor unused import warnings remain)

## Log Output Locations

### Backend Logs
- **Location:** `~/.config/sak-editor/logs/sak-editor_YYYY-MM-DD_HH-MM-SS.log`
- **Format:** Timestamped with `[component]` prefixes
- **Levels:** ERROR, WARN, INFO, DEBUG

### Frontend Logs
- **Location:** Browser console (captured by Playwright)
- **Format:** `[ComponentName] message`
- **Levels:** console.error, console.warn, console.info, console.debug

### Playwright Test Logs
- **Location:** Playwright test output and browser console
- **Format:** `[Test] message`
- Captures both frontend and test execution logs

## Usage

### View Backend Logs
```bash
./scripts/view-logs.sh -f        # Follow logs in real-time
./scripts/view-logs.sh -p        # Show only plugin logs
./scripts/view-logs.sh -e        # Show only errors
./scripts/view-logs.sh --clear   # Clear all logs
```

### Run Playwright Tests
```bash
npm run test:e2e -- tests/e2e/playwright/plugin-system.spec.ts
```

### Set Log Level
```bash
SAK_LOG_LEVEL=debug ./sak-editor
```

## Debugging the ACL Error

The comprehensive logging will help debug the "plugin:dialog|open not allowed by ACL" error by:

1. **Backend:** Logs all plugin system initialization and command invocations
2. **Frontend:** Logs all plugin API calls with parameters
3. **Tests:** Capture console output showing exactly when the error occurs

The log output will show:
- When plugin commands are invoked
- What parameters are passed
- Whether capabilities are found/executed
- Any errors returned from the backend
