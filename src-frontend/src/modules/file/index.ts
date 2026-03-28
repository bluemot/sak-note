import { uiRegistry } from '../../ui-system/ModuleUIRegistry';

export function registerFileModule() {
  uiRegistry.registerModule({
    module: 'file',
    version: '1.0',
    components: [
      {
        id: 'open',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'file',
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
        module: 'file',
        title: 'Save',
        icon: '💾',
        action: 'file:save',
        shortcut: 'Ctrl+S',
        order: 20,
        when: 'hasFileOpen'
      },
      {
        id: 'save-as',
        type: 'menu_item',
        slot: 'menu.file',
        module: 'file',
        title: 'Save As...',
        action: 'file:save_as',
        shortcut: 'Ctrl+Shift+S'
      }
    ],
    menus: [
      {
        id: 'file-menu',
        slot: 'menu.file',
        title: 'File',
        items: [
          { id: 'new', type: 'item', title: 'New File', action: 'file:new' },
          { id: 'sep1', type: 'separator' },
          { id: 'open', type: 'item', title: 'Open...', action: 'file:open', shortcut: 'Ctrl+O' },
          { id: 'open-recent', type: 'item', title: 'Open Recent', action: 'file:recent' },
          { id: 'sep2', type: 'separator' },
          { id: 'save', type: 'item', title: 'Save', action: 'file:save', shortcut: 'Ctrl+S' },
          { id: 'save-as', type: 'item', title: 'Save As...', action: 'file:save_as', shortcut: 'Ctrl+Shift+S' },
          { id: 'sep3', type: 'separator' },
          { id: 'close', type: 'item', title: 'Close', action: 'file:close' }
        ]
      }
    ],
    shortcuts: [
      { key: 's', modifiers: ['Ctrl'], action: 'file:save' },
      { key: 's', modifiers: ['Ctrl', 'Shift'], action: 'file:save_as' },
      { key: 'o', modifiers: ['Ctrl'], action: 'file:open' },
      { key: 'n', modifiers: ['Ctrl'], action: 'file:new' }
    ]
  });
}
