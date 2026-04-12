import { uiRegistry } from '../../ui-system/ModuleUIRegistry';

export function registerMarksModule() {
  uiRegistry.registerModule({
    module: 'marks',
    version: '1.0',
    components: [
      {
        id: 'highlight-selection',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'marks',
        title: 'Highlight',
        icon: '\u{1f3a8}',
        action: 'marks:highlight_selection',
        order: 71,
        when: 'hasFileOpen'
      },
      {
        id: 'create',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'marks',
        title: 'Create Mark',
        icon: '\u{1f4cd}',
        action: 'marks:create',
        shortcut: 'Ctrl+M',
        order: 70,
        when: 'hasFileOpen'
      },
      {
        id: 'panel',
        type: 'sidebar_panel',
        slot: 'sidebar.tabs',
        module: 'marks',
        title: '\u{1f4cd} Marks',
        icon: '\u{1f4cd}',
        order: 60
      }
    ],
    menus: [
      {
        id: 'marks-menu',
        slot: 'menu.tools',
        title: 'Marks',
        items: [
          { id: 'highlight', type: 'item', title: 'Highlight Selection', action: 'marks:highlight_selection' },
          { id: 'create', type: 'item', title: 'Create Mark', action: 'marks:create', shortcut: 'Ctrl+M' },
          { id: 'delete', type: 'item', title: 'Delete Mark', action: 'marks:delete' },
          { id: 'sep1', type: 'separator' },
          { id: 'clear', type: 'item', title: 'Clear All Marks', action: 'marks:clear' },
          { id: 'sep2', type: 'separator' },
          { id: 'export', type: 'item', title: 'Export Marks...', action: 'marks:export' },
          { id: 'import', type: 'item', title: 'Import Marks...', action: 'marks:import' }
        ]
      }
    ],
    shortcuts: [
      { key: 'm', modifiers: ['Ctrl'], action: 'marks:create' },
      { key: 'm', modifiers: ['Ctrl', 'Shift'], action: 'marks:delete' },
      { key: 'h', modifiers: ['Ctrl', 'Alt'], action: 'marks:highlight_selection' }
    ]
  });
}