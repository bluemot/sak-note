import { uiRegistry } from '../../ui-system/ModuleUIRegistry';

export function registerEditModule() {
  uiRegistry.registerModule({
    module: 'edit',
    version: '1.0',
    components: [
      {
        id: 'undo',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'edit',
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
        module: 'edit',
        title: 'Redo',
        icon: '↪️',
        action: 'edit:redo',
        shortcut: 'Ctrl+Y',
        order: 31,
        when: 'canRedo'
      },
      {
        id: 'find',
        type: 'toolbar_button',
        slot: 'toolbar.search',
        module: 'edit',
        title: 'Find',
        icon: '🔍',
        action: 'edit:find',
        shortcut: 'Ctrl+F',
        order: 10
      },
      {
        id: 'replace',
        type: 'toolbar_button',
        slot: 'toolbar.search',
        module: 'edit',
        title: 'Replace',
        icon: '🔁',
        action: 'edit:replace',
        shortcut: 'Ctrl+H',
        order: 20
      }
    ],
    menus: [
      {
        id: 'edit-menu',
        slot: 'menu.edit',
        title: 'Edit',
        items: [
          { id: 'undo', type: 'item', title: 'Undo', action: 'edit:undo', shortcut: 'Ctrl+Z' },
          { id: 'redo', type: 'item', title: 'Redo', action: 'edit:redo', shortcut: 'Ctrl+Y' },
          { id: 'sep1', type: 'separator' },
          { id: 'cut', type: 'item', title: 'Cut', action: 'edit:cut', shortcut: 'Ctrl+X' },
          { id: 'copy', type: 'item', title: 'Copy', action: 'edit:copy', shortcut: 'Ctrl+C' },
          { id: 'paste', type: 'item', title: 'Paste', action: 'edit:paste', shortcut: 'Ctrl+V' },
          { id: 'sep2', type: 'separator' },
          { id: 'find', type: 'item', title: 'Find', action: 'edit:find', shortcut: 'Ctrl+F' },
          { id: 'replace', type: 'item', title: 'Replace', action: 'edit:replace', shortcut: 'Ctrl+H' },
          { id: 'find-in-files', type: 'item', title: 'Find in Files', action: 'edit:find_in_files', shortcut: 'Ctrl+Shift+F' },
          { id: 'goto', type: 'item', title: 'Go to Line', action: 'edit:goto_line', shortcut: 'Ctrl+G' }
        ]
      }
    ],
    shortcuts: [
      { key: 'z', modifiers: ['Ctrl'], action: 'edit:undo' },
      { key: 'y', modifiers: ['Ctrl'], action: 'edit:redo' },
      { key: 'x', modifiers: ['Ctrl'], action: 'edit:cut' },
      { key: 'c', modifiers: ['Ctrl'], action: 'edit:copy' },
      { key: 'v', modifiers: ['Ctrl'], action: 'edit:paste' },
      { key: 'f', modifiers: ['Ctrl'], action: 'edit:find' },
      { key: 'h', modifiers: ['Ctrl'], action: 'edit:replace' },
      { key: 'g', modifiers: ['Ctrl'], action: 'edit:goto_line' }
    ]
  });
}
