# SAK Editor Plugin System

## Architecture Overview

A dynamic plugin system where third-party modules can be added without touching the core framework source code.

## Core Design Principles

1. **Configuration-Driven**: Plugins define their capabilities via JSON
2. **Hot-Loadable**: Plugins can be loaded/unloaded at runtime
3. **API-First**: Plugins use a stable API to interact with the editor
4. **Isolated**: Plugins run in isolation and can't crash the main editor

## Plugin Structure

```
plugins/
├── my-plugin/
│   ├── manifest.json       # Plugin metadata and capabilities
│   ├── plugin.wasm         # WebAssembly module (Rust compiled to WASM)
│   ├── ui/
│   │   ├── MyPanel.tsx     # React components
│   │   └── styles.css
│   └── assets/
│       └── icon.svg
```

## Plugin Manifest (manifest.json)

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "A sample plugin for SAK Editor",
  "author": "Your Name",
  "min_app_version": "0.1.0",
  "capabilities": {
    "commands": [
      {
        "name": "my_action",
        "description": "Performs my action",
        "params": {
          "input": { "type": "string", "required": true }
        }
      }
    ],
    "ui_components": [
      {
        "type": "sidebar_panel",
        "id": "my_panel",
        "title": "My Panel",
        "slot": "sidebar.tabs",
        "file": "ui/MyPanel.tsx"
      },
      {
        "type": "toolbar_button",
        "id": "my_button",
        "title": "My Button",
        "slot": "toolbar.main",
        "icon": "assets/icon.svg",
        "command": "my_action"
      }
    ],
    "menus": [
      {
        "slot": "menu.tools",
        "items": [
          {
            "id": "my_menu_item",
            "title": "My Tool",
            "command": "my_action",
            "shortcut": "Ctrl+Shift+M"
          }
        ]
      }
    ]
  },
  "permissions": [
    "filesystem:read",
    "filesystem:write",
    "editor:content",
    "editor:selection"
  ]
}
```

## Plugin API

### JavaScript API (for UI components)

```typescript
// In plugin's React component
import { usePluginAPI } from '@sak-editor/plugin-api'

function MyPanel() {
  const api = usePluginAPI()
  
  const handleClick = async () => {
    // Access editor state
    const currentFile = await api.editor.getCurrentFile()
    const content = await api.editor.getContent()
    
    // Modify editor
    await api.editor.insertText(currentFile, line, "Hello from plugin!")
    
    // Call plugin's Rust function
    const result = await api.invoke('my_action', { input: 'test' })
    
    // Show notification
    api.ui.showNotification({ message: 'Done!', type: 'success' })
  }
  
  return <button onClick={handleClick}>Run Plugin</button>
}
```

### Rust API (for plugin logic)

```rust
// In plugin's WASM module
use sak_editor_plugin_api::*;

#[plugin_entrypoint]
pub fn init() {
    // Register commands
    register_command("my_action", |params| {
        let input = params.get_str("input")?;
        let result = process_data(input);
        
        // Access editor
        let editor = api::get_editor();
        let selection = editor.get_selection()?;
        
        Ok(json!({ "result": result }))
    });
}

#[command]
pub fn process_data(input: String) -> String {
    format!("Processed: {}", input)
}
```

## Plugin Manager

The core PluginManager handles:

1. **Discovery**: Scan `plugins/` directory
2. **Validation**: Verify manifest and permissions
3. **Loading**: Load WASM modules
4. **UI Registration**: Register components dynamically
5. **Command Registration**: Register Tauri commands dynamically
6. **Lifecycle**: Handle plugin start/stop/update

## Implementation Strategy

Since Tauri uses static compilation, we need a hybrid approach:

### Option 1: WASM-based Plugins (Recommended)
- Compile plugin Rust code to WASM
- Load WASM at runtime using `wasmtime`
- WASM provides isolation and safety
- Dynamic loading without recompiling core

### Option 2: JavaScript/TypeScript Plugins
- Pure JS plugins that use the exposed API
- Loaded via dynamic import()
- Sandboxed execution
- Easier to develop but less powerful

### Option 3: Scripting Engine
- Embed Lua or JavaScript runtime (e.g., Deno)
- Scripts loaded from files at runtime
- Full flexibility but less type-safe

## Core Plugin System Components

### 1. PluginRegistry
```rust
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    command_handlers: HashMap<String, CommandHandler>,
    ui_components: HashMap<String, UIComponentDefinition>,
}

impl PluginRegistry {
    pub fn load_plugin(&mut self, path: &Path) -> Result<(), PluginError>;
    pub fn unload_plugin(&mut self, id: &str) -> Result<(), PluginError>;
    pub fn register_command(&mut self, plugin_id: &str, name: &str, handler: CommandHandler);
    pub fn invoke(&self, plugin_id: &str, command: &str, params: Value) -> Result<Value>;
}
```

### 2. Plugin Trait
```rust
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self, api: &PluginAPI) -> Result<(), PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
}
```

### 3. PluginAPI
```rust
pub struct PluginAPI {
    // Editor access
    pub editor: EditorAPI,
    // File system
    pub fs: FileSystemAPI,
    // UI
    pub ui: UIAPI,
    // Events
    pub events: EventBus,
}

pub struct EditorAPI {
    pub get_current_file: Box<dyn Fn() -> Option<String>>,
    pub get_content: Box<dyn Fn(&str) -> Result<String>>,
    pub set_content: Box<dyn Fn(&str, &str) -> Result<()>>,
    pub get_selection: Box<dyn Fn(&str) -> Result<Selection>>,
    pub insert_text: Box<dyn Fn(&str, usize, &str) -> Result<()>>,
}
```

## Security Model

1. **Permission System**: Plugins declare required permissions
2. **Capability-based API**: Plugins can only access granted capabilities
3. **Sandboxing**: WASM provides memory isolation
4. **Review Process**: Built-in plugin can prompt user for approval

## Loading Flow

1. User places plugin in `~/.config/sak-editor/plugins/`
2. PluginManager discovers plugin on startup or via manual refresh
3. Validate manifest.json schema and check compatibility
4. Load WASM module into isolated context
5. Call `plugin.init()` to register commands and UI
6. UI components are dynamically registered in ModuleUIRegistry
7. Commands are registered with Tauri (via proxy)

## Future Enhancements

- Plugin marketplace / repository
- Auto-update mechanism
- Plugin signing and verification
- Crash isolation (plugin crash doesn't affect main app)
- Performance profiling for plugins

## Implementation Status

🚧 **In Progress** - Phase 1: Core plugin system infrastructure
- [ ] Plugin discovery mechanism
- [ ] Manifest schema validation
- [ ] WASM runtime integration
- [ ] PluginAPI definition
- [ ] Dynamic UI registration
- [ ] Permission system
