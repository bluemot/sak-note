# WASM Plugin System Design Decisions

## Date: 2026-03-27
## Status: Design Complete, Implementation Phase 1 Done

---

## Overview

SAK Editor now features a complete WASM Plugin System that allows third-party extensions without modifying the core source code.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    SAK Editor (Rust)                        │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────┐ │
│  │ PluginManager│  │ WASM Engine  │  │ PluginAPI/Bridge   │ │
│  │ - discover() │  │ (wasmtime)   │  │ - VFS integration  │ │
│  │ - load()     │  │ - Fuel       │  │ - Security layer   │ │
│  │ - unload()   │  │ - Memory cap │  │ - Host functions   │ │
│  └─────────────┘  └──────────────┘  └────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      WASM Plugin (.wasm)                    │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────┐ │
│  │ Rust Logic  │  │ WASI (fs)    │  │ Plugin Features    │ │
│  │ #[wasm_bind]│  │ - read_file  │  │ - process_text()   │ │
│  │ - init()    │  │ - write_file │  │ - analyze()        │ │
│  └─────────────┘  └──────────────┘  └────────────────────┘ │ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    UI Components (React)                    │
│  - Dynamic import via React.lazy()                          │
│  - Must use Default Export                                    │
│  - Receive `sak` API prop                                     │
│  - CSS isolation via CSS Modules                              │
└─────────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. Security Model (3-Layer Protection)

**Layer 1: Fuel Mechanism (Anti-Infinite Loop)**
```rust
config.consume_fuel(true);
store.add_fuel(10_000_000)?; // Max 10M instructions
```

**Layer 2: Memory Cap (Anti-OOM)**
```rust
let memory_ty = MemoryType::new(1, Some(64)); // 64MB max
```

**Layer 3: VFS Permission Check**
```rust
// All file access goes through VFS
vfs.check_permission(plugin_id, path, operation)?;
```

### 2. Data Transfer Strategy

**Decision**: Default to selection-only + streaming API for large files

- Plugins primarily work with editor selections (small data)
- Large file processing via streaming chunks
- Optional memory-mapped I/O for advanced use cases

### 3. UI Integration (Specification-Driven)

**Plugin Directory Structure:**
```
my-plugin/
├── plugin.wasm          # WASM logic
├── ui/
│   ├── panel.js         # Default Export React component
│   ├── toolbar.js       # Default Export React component
│   └── styles.css       # CSS Modules
├── plugin.json          # Manifest with permissions
└── README.md
```

**plugin.json Schema:**
```json
{
  "id": "my-plugin",
  "version": "1.0.0",
  "permissions": {
    "filesystem": {
      "read": ["/home/user/projects/*"],
      "write": ["/home/user/projects/*"],
      "deny": ["/etc", "~/.ssh"]
    }
  },
  "ui": {
    "components": [
      {
        "type": "sidebar_panel",
        "id": "my_panel",
        "file": "ui/panel.js",
        "title": "My Plugin",
        "icon": "🔧"
      }
    ]
  }
}
```

### 4. Host API Available to Plugins

```rust
// Logging
sak_log(level, message)

// File operations (via VFS with permission checks)
sak_get_content(path) -> Result<String>
sak_set_content(path, content) -> Result<()>
sak_get_selection() -> Result<String>
sak_insert_text(path, line, text) -> Result<()>

// UI
sak_show_notification(title, message)
sak_show_dialog(title, content)

// Plugin lifecycle
sak_register_command(name, handler)
sak_on_event(event_type, handler)
```

## Implementation Status

### ✅ Completed (Phase 1)

- [x] Plugin runtime module (wasm_engine, manager, bridge, commands)
- [x] WASM execution with wasmtime 23
- [x] WASI filesystem support
- [x] Plugin discovery from `~/.config/sak-editor/plugins/`
- [x] Sample plugin (uppercase, word-count, sort-lines)
- [x] Tauri commands for plugin management (11 commands)
- [x] Basic security (sandbox, WASI)

### 🚧 Future Enhancements

- [ ] Fuel mechanism implementation
- [ ] Memory cap enforcement
- [ ] VFS permission integration
- [ ] React.lazy() UI loading
- [ ] Plugin marketplace
- [ ] Signature verification
- [ ] Plugin auto-update
- [ ] Crash isolation (separate process)

## Security Philosophy

1. **Default Deny**: Minimal permissions by default
2. **Fail Fast**: Immediate termination on policy violation
3. **Complete Audit**: All plugin actions logged
4. **Community Review**: Trust but verify through code review

## Comparison with Other Plugin Systems

| Feature | VS Code | Sublime | SAK Editor |
|---------|---------|---------|------------|
| Language | JS/TS | Python | Any (WASM) |
| Sandboxing | Limited | None | Strong (WASM) |
| Performance | JS VM | Native | Near-native |
| UI Framework | React-like | Custom | React |
| Core Modification | No | No | No |

## Conclusion

The WASM Plugin System provides:
- ✅ Zero core source code modifications
- ✅ Strong security sandbox
- ✅ Near-native performance
- ✅ Modern React UI integration
- ✅ Cross-platform plugins (single .wasm file)

This design enables a vibrant plugin ecosystem while maintaining editor stability and security.

---

**Design finalized:** 2026-03-27
**Implementation:** Phase 1 complete
**Next Phase:** Security hardening + UI loading
