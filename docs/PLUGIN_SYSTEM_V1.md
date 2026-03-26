# SAK Editor Plugin System V1 - JavaScript Plugin API

## 快速開始

這個版本使用 **JavaScript/TypeScript 插件**，不需要 WASM 或重新編譯核心！

## 如何建立插件

### 1. 建立插件目錄

```bash
mkdir -p ~/.config/sak-editor/plugins/my-plugin
cd ~/.config/sak-editor/plugins/my-plugin
```

### 2. 建立插件配置 (`plugin.json`)

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "entry": "index.js",
  "ui": {
    "components": [
      {
        "type": "sidebar_panel",
        "id": "my_panel",
        "title": "🔧 My Plugin",
        "slot": "sidebar.tabs"
      }
    ],
    "commands": [
      {
        "id": "my_action",
        "title": "My Action",
        "shortcut": "Ctrl+Shift+M"
      }
    ]
  }
}
```

### 3. 建立插件程式碼 (`index.js`)

```javascript
// Plugin entry point
export default function initPlugin(sak) {
  // Register command
  sak.registerCommand('my_action', async (params) => {
    // Access editor
    const currentFile = sak.editor.getCurrentFile();
    const content = await sak.editor.getContent(currentFile);
    
    // Do something
    const lines = content.split('\n');
    const result = `Lines: ${lines.length}`;
    
    // Show notification
    sak.ui.showNotification(result);
    
    return { success: true, data: result };
  });
  
  // Register UI component
  sak.registerComponent('my_panel', MyPanel);
  
  console.log('My Plugin loaded!');
}

// React component
function MyPanel() {
  const handleClick = async () => {
    await sak.invoke('my_action', {});
  };
  
  return (
    React.createElement('div', null,
      React.createElement('h3', null, 'My Plugin'),
      React.createElement('button', { onClick: handleClick }, 'Run')
    )
  );
}
```

## 核心需要實作

### 1. PluginLoader (在 core 中)

```typescript
// src-frontend/src/core/PluginLoader.ts

interface Plugin {
  id: string;
  name: string;
  version: string;
  entry: string;
  manifest: PluginManifest;
}

class PluginLoader {
  private plugins: Map<string, Plugin> = new Map();
  private sakAPI: SakAPI;
  
  async discoverPlugins(): Promise<Plugin[]> {
    // Scan ~/.config/sak-editor/plugins/*/
    const pluginDirs = await this.scanPluginDirectory();
    
    const plugins: Plugin[] = [];
    for (const dir of pluginDirs) {
      const manifest = await this.loadManifest(dir);
      if (manifest) {
        plugins.push({
          id: manifest.id,
          name: manifest.name,
          version: manifest.version,
          entry: `${dir}/${manifest.entry}`,
          manifest
        });
      }
    }
    
    return plugins;
  }
  
  async loadPlugin(plugin: Plugin): Promise<void> {
    try {
      // Dynamic import of plugin JS
      const pluginModule = await import(/* @vite-ignore */ plugin.entry);
      
      // Initialize plugin with API
      if (pluginModule.default) {
        await pluginModule.default(this.sakAPI);
      }
      
      this.plugins.set(plugin.id, plugin);
      console.log(`Plugin loaded: ${plugin.name} v${plugin.version}`);
      
    } catch (error) {
      console.error(`Failed to load plugin ${plugin.id}:`, error);
    }
  }
  
  async loadAllPlugins(): Promise<void> {
    const plugins = await this.discoverPlugins();
    for (const plugin of plugins) {
      await this.loadPlugin(plugin);
    }
  }
}
```

### 2. SakAPI (提供給插件使用)

```typescript
// src-frontend/src/core/SakAPI.ts

export class SakAPI {
  constructor(
    private editorAPI: EditorAPI,
    private uiAPI: UIAPI,
    private tauriAPI: typeof import('@tauri-apps/api/core')
  ) {}
  
  // Editor API
  editor = {
    getCurrentFile: () => this.editorAPI.getCurrentFile(),
    getContent: (path: string) => this.editorAPI.getContent(path),
    setContent: (path: string, content: string) => this.editorAPI.setContent(path, content),
    getSelection: () => this.editorAPI.getSelection(),
    insertText: (path: string, line: number, text: string) => 
      this.editorAPI.insertText(path, line, text),
    replaceRange: (path: string, start: number, end: number, text: string) =>
      this.editorAPI.replaceRange(path, start, end, text),
  };
  
  // UI API
  ui = {
    showNotification: (message: string, type?: 'info' | 'success' | 'error') =>
      this.uiAPI.showNotification(message, type),
    showDialog: (title: string, content: React.ReactNode) =>
      this.uiAPI.showDialog(title, content),
    registerComponent: (id: string, component: React.ComponentType) =>
      this.uiAPI.registerComponent(id, component),
  };
  
  // Commands
  registerCommand = (id: string, handler: (params: any) => Promise<any>) => {
    CommandRegistry.register(`plugin:${this.currentPluginId}:${id}`, handler);
  };
  
  invoke = async (command: string, params: any) => {
    return await this.tauriAPI.invoke(command, params);
  };
}
```

## 優勢

1. **無需重新編譯**: 插件是純 JavaScript，核心不需要重新編譯
2. **動態載入**: 放置插件檔案後重啟即可載入
3. **安全**: 插件在瀏覽器/渲染進程中運行，與 Rust 後端隔離
4. **簡單**: 使用熟悉的 JavaScript/TypeScript 開發

## 限制

1. **不能直接調用系統 API**: 必須通過 Tauri 提供的 invoke
2. **效能**: 比 WASM 或原生 Rust 慢（但對 UI 插件足夠）
3. **類型安全**: JavaScript 沒有編譯時類型檢查

## 進階：TypeScript 支援

```typescript
// types/sak-editor.d.ts

declare module 'sak-editor' {
  export interface EditorAPI {
    getCurrentFile(): string | null;
    getContent(path: string): Promise<string>;
    setContent(path: string, content: string): Promise<void>;
    getSelection(): Promise<Selection>;
    insertText(path: string, line: number, text: string): Promise<void>;
  }
  
  export interface UIAPI {
    showNotification(message: string, type?: 'info' | 'success' | 'error'): void;
    showDialog(title: string, content: React.ReactNode): void;
  }
  
  export interface SakAPI {
    editor: EditorAPI;
    ui: UIAPI;
    registerCommand(id: string, handler: (params: any) => Promise<any>): void;
    invoke(command: string, params: any): Promise<any>;
  }
  
  export default function initPlugin(sak: SakAPI): void;
}
```

## 下一步

未來可以進化到 V2 (WASM 插件) 或 V3 (完整插件市場)。
