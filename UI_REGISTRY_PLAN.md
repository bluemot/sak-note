# SAK Editor - UI Registry Pattern 規劃方案

## 現況分析

### 已有基礎
✅ `src-frontend/src/ui-system/ModuleUIRegistry.ts` - 完整的 JSON-driven UI Registry 系統

### 問題
❌ `src-frontend/src/modules/` 目錄不存在 - 各模組沒有獨立的前端註冊檔案
❌ Toolbar.tsx / Sidebar.tsx / Menu - 還是 Hardcode 寫法，沒使用 Registry

---

## 目標架構

```
src-frontend/src/
├── ui-system/
│   ├── ModuleUIRegistry.ts       ✅ 已存在
│   ├── hooks/
│   │   ├── useSlotComponents.ts  - 讀取 slot 組件
│   │   ├── useUIAction.ts        - 執行 action
│   │   └── useRegistry.ts        - Registry 狀態
│   ├── components/
│   │   ├── DynamicToolbar.tsx    - 動態 Toolbar
│   │   ├── DynamicSidebar.tsx    - 動態 Sidebar
│   │   ├── DynamicMenu.tsx       - 動態 Menu
│   │   └── DynamicStatusBar.tsx  - 動態 StatusBar
│   └── actions/
│       ├── actionHandlers.ts     - Action 處理器
│       └── commandBridge.ts      - Tauri Command 橋接
│
├── modules/                        ⭐ 新增
│   ├── file/
│   │   ├── index.ts              - File Module 註冊
│   │   ├── actions.ts            - File 相關 actions
│   │   └── components/
│   │       └── RecentFilesMenu.tsx
│   ├── sftp/
│   │   ├── index.ts              - SFTP Module 註冊
│   │   ├── actions.ts            - SFTP 相關 actions
│   │   └── components/
│   │       └── SftpSitePanel.tsx   ✅ 已存在，需要整合
│   ├── edit/
│   │   ├── index.ts              - Edit Module 註冊
│   │   └── actions.ts            - Undo/Redo/Search
│   ├── marks/
│   │   ├── index.ts              - Marks Module 註冊
│   │   ├── actions.ts            - Create/Delete Mark
│   │   └── components/
│   │       └── MarkPanel.tsx       ✅ 已存在，需要整合
│   ├── bookmark/
│   │   ├── index.ts              - Bookmark Module 註冊
│   │   ├── actions.ts
│   │   └── components/
│   │       └── BookmarkPanel.tsx     ✅ 已存在，需要整合
│   ├── llm/
│   │   ├── index.ts              - LLM Module 註冊
│   │   ├── actions.ts            - Chat/Summarize
│   │   └── components/
│   │       └── LlmChat.tsx           ✅ 已存在，需要整合
│   ├── print/
│   │   ├── index.ts              - Print Module 註冊
│   │   └── actions.ts            - Print/Export PDF
│   └── plugin/
│       ├── index.ts              - Plugin Module 註冊
│       └── actions.ts            - Plugin Manager
│
└── App.tsx
    └── 初始化: registerAllModules()
```

---

## Phase 1: 核心架構 (2-3 天)

### 1.1 建立 Module 基礎結構

```typescript
// src-frontend/src/modules/types.ts
export interface ModuleConfig {
  id: string;
  name: string;
  version: string;
  register: () => void;
  unregister?: () => void;
}

export interface ModuleAction {
  id: string;
  handler: (params?: any) => Promise<any>;
}
```

### 1.2 建立 Action 處理系統

```typescript
// src-frontend/src/ui-system/actions/actionRegistry.ts
class ActionRegistry {
  private handlers: Map<string, Function> = new Map();
  
  register(moduleId: string, actionId: string, handler: Function) {
    this.handlers.set(`${moduleId}:${actionId}`, handler);
  }
  
  async execute(actionId: string, params?: any) {
    const handler = this.handlers.get(actionId);
    if (handler) {
      return await handler(params);
    }
    console.error(`Action ${actionId} not found`);
  }
}

export const actionRegistry = new ActionRegistry();
```

### 1.3 建立動態組件系統

```typescript
// src-frontend/src/ui-system/components/DynamicToolbar.tsx
export const DynamicToolbar: React.FC = () => {
  const items = useSlotComponents('toolbar.main');
  
  return (
    <div className="toolbar">
      {items.map(item => (
        <ToolbarButton
          key={item.id}
          icon={item.icon}
          tooltip={item.tooltip}
          onClick={() => executeAction(item.action)}
        />
      ))}
    </div>
  );
};
```

---

## Phase 2: Module 實作 (4-5 天)

### 2.1 File Module

```typescript
// src-frontend/src/modules/file/index.ts
export function registerFileModule() {
  // Register UI components
  uiRegistry.registerModule({
    module: 'file',
    version: '1.0',
    components: [
      {
        id: 'open',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        title: 'Open',
        icon: '📂',
        action: 'file:open',
        shortcut: 'Ctrl+O',
        order: 10
      },
      {
        id: 'save',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        title: 'Save',
        icon: '💾',
        action: 'file:save',
        shortcut: 'Ctrl+S',
        order: 20,
        when: 'hasFileOpen'  // 條件顯示
      },
      {
        id: 'save-as',
        type: 'menu_item',
        slot: 'menu.file',
        title: 'Save As...',
        action: 'file:save_as',
        shortcut: 'Ctrl+Shift+S'
      },
      {
        id: 'recent-files',
        type: 'menu_submenu',
        slot: 'menu.file',
        title: 'Open Recent',
        action: 'file:recent_menu'
      }
    ],
    shortcuts: [
      { key: 's', modifiers: ['Ctrl'], action: 'file:save' },
      { key: 's', modifiers: ['Ctrl', 'Shift'], action: 'file:save_as' },
      { key: 'o', modifiers: ['Ctrl'], action: 'file:open' },
      { key: 'n', modifiers: ['Ctrl'], action: 'file:new' }
    ]
  });
  
  // Register actions
  actionRegistry.register('file', 'open', async () => {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({ multiple: false });
    if (selected) {
      await invoke('open_file', { path: selected });
    }
  });
  
  actionRegistry.register('file', 'save', async () => {
    await invoke('save_file', { path: currentFilePath });
  });
  
  actionRegistry.register('file', 'save_as', async () => {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const path = await save();
    if (path) {
      await invoke('save_as', { source_path: currentFilePath, target_path: path });
    }
  });
}
```

### 2.2 Edit Module

```typescript
// src-frontend/src/modules/edit/index.ts
export function registerEditModule() {
  uiRegistry.registerModule({
    module: 'edit',
    version: '1.0',
    components: [
      {
        id: 'undo',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        title: 'Undo',
        icon: '↩️',
        action: 'edit:undo',
        shortcut: 'Ctrl+Z',
        order: 30,
        when: 'canUndo'
      },
      {
        id: 'redo',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        title: 'Redo',
        icon: '↪️',
        action: 'edit:redo',
        shortcut: 'Ctrl+Y',
        order: 31,
        when: 'canRedo'
      },
      {
        id: 'search-panel',
        type: 'panel',
        slot: 'panel.bottom',
        title: 'Search Results',
        component: 'SearchPanel',
        visible: false  // 預設隱藏
      }
    ],
    menus: [
      {
        id: 'edit-menu',
        slot: 'menu.edit',
        title: 'Edit',
        items: [
          { id: 'undo', type: 'item', title: 'Undo', action: 'edit:undo' },
          { id: 'redo', type: 'item', title: 'Redo', action: 'edit:redo' },
          { id: 'sep1', type: 'separator' },
          { id: 'cut', type: 'item', title: 'Cut', action: 'edit:cut' },
          { id: 'copy', type: 'item', title: 'Copy', action: 'edit:copy' },
          { id: 'paste', type: 'item', title: 'Paste', action: 'edit:paste' },
          { id: 'sep2', type: 'separator' },
          { id: 'find', type: 'item', title: 'Find', action: 'edit:find' },
          { id: 'replace', type: 'item', title: 'Replace', action: 'edit:replace' },
          { id: 'find-in-files', type: 'item', title: 'Find in Files', action: 'edit:find_in_files' },
          { id: 'sep3', type: 'separator' },
          { id: 'goto', type: 'item', title: 'Go to Line', action: 'edit:goto_line' },
          { id: 'sep4', type: 'separator' },
          { 
            id: 'bookmarks', 
            type: 'submenu', 
            title: 'Bookmarks',
            submenu: [
              { id: 'toggle', type: 'item', title: 'Toggle Bookmark', action: 'bookmark:toggle' },
              { id: 'next', type: 'item', title: 'Next Bookmark', action: 'bookmark:next' },
              { id: 'prev', type: 'item', title: 'Previous Bookmark', action: 'bookmark:prev' },
              { id: 'clear', type: 'item', title: 'Clear All Bookmarks', action: 'bookmark:clear' }
            ]
          }
        ]
      }
    ]
  });
  
  // Actions
  actionRegistry.register('edit', 'undo', () => invoke('undo', { path: currentFile }));
  actionRegistry.register('edit', 'redo', () => invoke('redo', { path: currentFile }));
  actionRegistry.register('edit', 'find', () => showSearchPanel());
  actionRegistry.register('edit', 'goto_line', () => showGoToLineDialog());
}
```

### 2.3 SFTP Module

```typescript
// src-frontend/src/modules/sftp/index.ts
export function registerSftpModule() {
  uiRegistry.registerModule({
    module: 'sftp',
    version: '1.0',
    components: [
      {
        id: 'open-remote',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        title: 'SFTP',
        icon: '🌐',
        tooltip: 'Open Remote File (SFTP)',
        action: 'sftp:open_remote',
        order: 100
      },
      {
        id: 'sftp-sites',
        type: 'sidebar_panel',
        slot: 'sidebar.tabs',
        title: '🌐 SFTP Sites',
        icon: 'globe',
        component: 'SftpSitePanel',
        order: 50
      }
    ],
    menus: [
      {
        id: 'file-sftp',
        slot: 'menu.file',
        items: [
          { id: 'open-remote', type: 'item', title: 'Open Remote (SFTP)', action: 'sftp:open_remote' },
          { id: 'sftp-manager', type: 'item', title: 'SFTP Site Manager', action: 'sftp:site_manager' }
        ]
      }
    ]
  });
  
  actionRegistry.register('sftp', 'open_remote', async () => {
    // 顯示 SFTP 連線對話框
    await invoke('sftp_connect_dialog');
  });
  
  actionRegistry.register('sftp', 'site_manager', async () => {
    // 顯示 Site Manager
    showSftpSiteManager();
  });
}
```

### 2.4 Marks Module

```typescript
// src-frontend/src/modules/marks/index.ts
export function registerMarksModule() {
  uiRegistry.registerModule({
    module: 'marks',
    version: '1.0',
    components: [
      {
        id: 'marks-panel',
        type: 'sidebar_panel',
        slot: 'sidebar.tabs',
        title: '🎨 Marks',
        icon: 'highlighter',
        component: 'MarkPanel',
        order: 40
      }
    ],
    menus: [
      {
        id: 'edit-marks',
        slot: 'menu.edit',
        items: [
          { 
            id: 'marks-submenu', 
            type: 'submenu', 
            title: 'Color Marks',
            submenu: [
              { id: 'mark-red', type: 'item', title: 'Red Mark', action: 'marks:create_red' },
              { id: 'mark-yellow', type: 'item', title: 'Yellow Mark', action: 'marks:create_yellow' },
              { id: 'mark-green', type: 'item', title: 'Green Mark', action: 'marks:create_green' },
              { id: 'sep', type: 'separator' },
              { id: 'clear-marks', type: 'item', title: 'Clear All Marks', action: 'marks:clear' },
              { id: 'export-marks', type: 'item', title: 'Export Marks', action: 'marks:export' }
            ]
          }
        ]
      }
    ],
    shortcuts: [
      { key: '1', modifiers: ['Ctrl', 'Shift'], action: 'marks:create_red' },
      { key: '2', modifiers: ['Ctrl', 'Shift'], action: 'marks:create_yellow' },
      { key: '3', modifiers: ['Ctrl', 'Shift'], action: 'marks:create_green' }
    ]
  });
}
```

### 2.5 其他 Modules

類似結構實作：
- `bookmark/index.ts`
- `llm/index.ts`
- `print/index.ts`
- `plugin/index.ts`

---

## Phase 3: 整合現有組件 (2-3 天)

### 3.1 修改 App.tsx

```typescript
// src-frontend/src/App.tsx
import { useEffect } from 'react';
import { registerFileModule } from './modules/file';
import { registerEditModule } from './modules/edit';
import { registerSftpModule } from './modules/sftp';
import { registerMarksModule } from './modules/marks';
import { registerBookmarkModule } from './modules/bookmark';
import { registerLlmModule } from './modules/llm';
import { registerPrintModule } from './modules/print';
import { DynamicToolbar } from './ui-system/components/DynamicToolbar';
import { DynamicSidebar } from './ui-system/components/DynamicSidebar';
import { DynamicMenuBar } from './ui-system/components/DynamicMenuBar';
import { DynamicStatusBar } from './ui-system/components/DynamicStatusBar';

function App() {
  useEffect(() => {
    // Register all modules on startup
    registerFileModule();
    registerEditModule();
    registerSftpModule();
    registerMarksModule();
    registerBookmarkModule();
    registerLlmModule();
    registerPrintModule();
    
    console.log('[App] All modules registered');
  }, []);
  
  return (
    <div className="app">
      <DynamicMenuBar />      {/* ⭐ 替換原有結構 */}
      <DynamicToolbar />      {/* ⭐ 動態 Toolbar */}
      <div className="main-container">
        <DynamicSidebar />    {/* ⭐ 動態 Sidebar */}
        <Editor />
      </div>
      <DynamicStatusBar />    {/* ⭐ 動態 StatusBar */}
    </div>
  );
}
```

### 3.2 逐步遷移現有組件

```typescript
// 在 module/index.ts 中引用現有組件
import { SftpSitePanel } from '../../components/SftpSiteManager';
import { MarkPanel } from '../../components/MarkPanel';
import { BookmarkPanel } from '../../components/BookmarkPanel';
import { LlmChat } from '../../components/LlmChat';

// 這樣不需要重寫組件，只是改變註冊方式
```

---

## Phase 4: 進階功能 (2-3 天)

### 4.1 條件顯示系統

```typescript
// src-frontend/src/ui-system/conditionEngine.ts
export class ConditionEngine {
  private conditions: Map<string, () => boolean> = new Map();
  
  register(name: string, evaluator: () => boolean) {
    this.conditions.set(name, evaluator);
  }
  
  evaluate(condition: string): boolean {
    const evaluator = this.conditions.get(condition);
    return evaluator ? evaluator() : true;
  }
}

// 註冊條件
conditionEngine.register('hasFileOpen', () => !!currentFile);
conditionEngine.register('canUndo', () => canUndo);
conditionEngine.register('canRedo', () => canRedo);
conditionEngine.register('hasSelection', () => hasSelection);
```

### 4.2 插件動態註冊

```typescript
// 支持 plugins 動態註冊 UI
export function registerPluginModule(pluginId: string, pluginUI: ModuleUIRegistration) {
  uiRegistry.registerModule(pluginUI);
  console.log(`[Plugin] ${pluginId} registered UI`);
}
```

### 4.3 快速鍵系統

```typescript
// src-frontend/src/ui-system/ShortcutManager.ts
export class ShortcutManager {
  registerShortcuts(shortcuts: ShortcutDefinition[]) {
    shortcuts.forEach(s => {
      const keyCombo = [...s.modifiers, s.key].join('+');
      Mousetrap.bind(keyCombo, () => {
        actionRegistry.execute(s.action);
        return false;
      });
    });
  }
}
```

---

## ⚠️ 實作時需要注意的「坑」(Technical Gotchas)

### 1. 條件引擎 (Condition Engine) 的 React 響應性問題

**問題**: 純 TypeScript 的 ConditionEngine 不會觸發 React re-render。

```typescript
// ❌ 錯誤做法：純類別，React 不會響應
class ConditionEngine {
  evaluate(condition: string): boolean {
    // 狀態變化時，React 不會重新渲染
  }
}
```

**✅ 建議做法**: 使用 Zustand + Custom Hook

```typescript
// src-frontend/src/store/uiStore.ts
import { create } from 'zustand';

interface UIState {
  hasFileOpen: boolean;
  canUndo: boolean;
  canRedo: boolean;
  hasSelection: boolean;
  currentFilePath: string | null;
  
  // Actions
  setFileOpen: (path: string | null) => void;
  setUndoRedo: (canUndo: boolean, canRedo: boolean) => void;
  setSelection: (hasSelection: boolean) => void;
}

export const useUIStore = create<UIState>((set) => ({
  hasFileOpen: false,
  canUndo: false,
  canRedo: false,
  hasSelection: false,
  currentFilePath: null,
  
  setFileOpen: (path) => set({ 
    hasFileOpen: !!path, 
    currentFilePath: path 
  }),
  setUndoRedo: (canUndo, canRedo) => set({ canUndo, canRedo }),
  setSelection: (hasSelection) => set({ hasSelection }),
}));

// src-frontend/src/ui-system/hooks/useCondition.ts
import { useUIStore } from '../../store/uiStore';

const conditionMap = {
  hasFileOpen: () => useUIStore.getState().hasFileOpen,
  canUndo: () => useUIStore.getState().canUndo,
  canRedo: () => useUIStore.getState().canRedo,
  hasSelection: () => useUIStore.getState().hasSelection,
};

export function useCondition(condition: string): boolean {
  // 這樣會在狀態變化時自動響應
  const value = useUIStore((state) => {
    const evaluator = conditionMap[condition as keyof typeof conditionMap];
    return evaluator ? evaluator() : true;
  });
  
  return value;
}

// 使用在組件中
function ToolbarButton({ item }: { item: UIComponentDefinition }) {
  const isVisible = useCondition(item.when || 'true');
  
  if (!isVisible) return null;
  
  return <button>{item.title}</button>;
}
```

---

### 2. 全域狀態的存取 (Shared State Access)

**問題**: Action Handler 需要知道 currentFilePath，但不能硬編碼。

```typescript
// ❌ 錯誤做法：硬編碼變數
actionRegistry.register('file', 'save', async () => {
  await invoke('save_file', { path: currentFilePath }); // currentFilePath 從哪來？
});
```

**✅ 建議做法**: 從 Store 讀取 或 參數傳遞

```typescript
// Option 1: 從 Store 讀取（推薦）
actionRegistry.register('file', 'save', async () => {
  const { currentFilePath } = useUIStore.getState();
  if (!currentFilePath) {
    throw new Error('No file open');
  }
  await invoke('save_file', { path: currentFilePath });
});

// Option 2: 參數傳遞
actionRegistry.register('file', 'save', async (params?: { path?: string }) => {
  const path = params?.path || useUIStore.getState().currentFilePath;
  if (!path) throw new Error('No file open');
  await invoke('save_file', { path });
});

// 使用時
executeAction('file:save'); // 自動從 Store 取 path
// 或
executeAction('file:save', { path: '/custom/path.txt' }); // 指定 path
```

---

### 3. Tauri API 的錯誤處理與通知系統

**問題**: 各 Module 都需要寫錯誤處理，程式碼重複。

**✅ 建議做法**: ActionRegistry 統一包裝

```typescript
// src-frontend/src/ui-system/actions/actionRegistry.ts
import { useUIStore } from '../../store/uiStore';

class ActionRegistry {
  private handlers: Map<string, Function> = new Map();
  
  register(moduleId: string, actionId: string, handler: Function) {
    this.handlers.set(`${moduleId}:${actionId}`, handler);
  }
  
  async execute(actionId: string, params?: any): Promise<any> {
    const handler = this.handlers.get(actionId);
    
    if (!handler) {
      console.error(`Action ${actionId} not found`);
      showNotification({
        type: 'error',
        message: `Action "${actionId}" not implemented`,
      });
      throw new Error(`Action ${actionId} not found`);
    }
    
    try {
      // 顯示 Loading 狀態
      useUIStore.getState().setLoading(true);
      
      const result = await handler(params);
      
      // 成功通知
      showNotification({
        type: 'success',
        message: `${actionId} completed`,
        duration: 2000,
      });
      
      return result;
      
    } catch (error) {
      // 統一錯誤處理
      console.error(`[Action Error] ${actionId}:`, error);
      
      showNotification({
        type: 'error',
        message: error instanceof Error ? error.message : 'Unknown error',
        duration: 5000,
      });
      
      throw error;
      
    } finally {
      useUIStore.getState().setLoading(false);
    }
  }
}

// 通知系統（簡單實現）
interface Notification {
  id: string;
  type: 'success' | 'error' | 'info';
  message: string;
  duration?: number;
}

export function showNotification(notif: Omit<Notification, 'id'>) {
  const notification = {
    ...notif,
    id: Date.now().toString(),
  };
  
  // 可以整合 Toast 組件
  console.log(`[Notification] ${notif.type}: ${notif.message}`);
  
  // 或發送給通知中心
  useUIStore.getState().addNotification(notification);
}
```

---

### 4. 額外注意事項

#### 4.1 組件 Lazy Loading

```typescript
// 避免一次性載入所有 Module 組件
const SftpSitePanel = lazy(() => import('../modules/sftp/components/SftpSitePanel'));

// 使用 Suspense
<Suspense fallback={<Loading />}>
  <DynamicSidebar />
</Suspense>
```

#### 4.2 Action ID 命名規範

```typescript
// 建議格式: module:action
'file:open'
'file:save'
'edit:undo'
'sftp:connect'
'marks:create_red'

// 避免重複
// ❌ 'open' (太泛)
// ✅ 'file:open' (明確)
```

#### 4.3 Module 卸載清理

```typescript
export function registerFileModule() {
  uiRegistry.registerModule(fileModuleConfig);
  
  // 返回清理函數
  return () => {
    uiRegistry.unregisterModule('file');
    actionRegistry.unregisterAll('file');
  };
}
```

#### 4.4 TypeScript 類型安全

```typescript
// 定義所有可能的 Action
export type FileAction = 
  | { type: 'file:open'; params?: { multiple?: boolean } }
  | { type: 'file:save'; params?: { path?: string } }
  | { type: 'file:close'; params?: never };

export type AppAction = FileAction | EditAction | SftpAction;

// 使用時有類型提示
executeAction('file:open', { multiple: true }); // ✅ 正確
executeAction('file:open', { wrongParam: true }); // ❌ 編譯錯誤
```

---

## 修正後的完整工時估計

加入以上細節處理後：

| Phase | 內容 | 工時 |
|-------|------|------|
| Phase 1 | 核心架構 + Store + Condition Hook | 3-4 天 |
| Phase 2 | Module 實作 + 錯誤處理 | 5-6 天 |
| Phase 3 | 整合現有組件 + 測試 | 3-4 天 |
| Phase 4 | 進階功能 + 通知系統 | 3-4 天 |
| **總計** | | **14-18 天** |
