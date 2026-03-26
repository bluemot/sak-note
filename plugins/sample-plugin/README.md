# Sample SAK Editor Plugin

A demonstration plugin showing how to build WASM plugins for SAK Editor.

## Capabilities

- **Uppercase**: Convert selected text to uppercase
- **Word Count**: Count words and characters in the document
- **Sort Lines**: Sort lines alphabetically or by length

## Building

```bash
# Install wasm target (first time only)
rustup target add wasm32-unknown-unknown

# Build the plugin
cargo build --release --target wasm32-unknown-unknown
```

The compiled WASM file will be at:
`target/wasm32-unknown-unknown/release/sample_plugin.wasm`

## Installing

1. Copy the `plugin.json` and compiled `.wasm` file to your SAK Editor plugins directory:
   - Linux/macOS: `~/.config/sak-editor/plugins/sample-plugin/`
   - Windows: `%APPDATA%\sak-editor\plugins\sample-plugin\`

2. Restart SAK Editor or use the plugin reload command

## Plugin API

Plugins can call these host functions:

- `sak_log` - Log messages to the editor console
- `sak_get_content` - Get file content
- `sak_set_content` - Set file content
- `sak_show_notification` - Show notifications
- `sak_get_setting` / `sak_set_setting` - Plugin settings

Plugins can export these functions:

- `__initialize` - Called when plugin loads
- `__shutdown` - Called when plugin unloads
- `__on_event` - Handle editor events
- `__capability_<id>` - Execute a capability