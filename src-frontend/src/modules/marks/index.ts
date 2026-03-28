import { uiRegistry } from '../../ui-system/ModuleUIRegistry';

export function registerMarksModule() {
  uiRegistry.registerModule({
    module: 'marks',
    version: '1.0',
    components: [
      {
        id: 'create',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'marks',
        title: 'Create Mark',
        icon: '📍',
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
        title: '📍 Marks',
        icon: '📍',
        order: 60
      }
    ],
    menus: [
      {
        id: 'marks-menu',
        slot: 'menu.tools',
        title: 'Marks',
        items: [
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
      { key: 'm', modifiers: ['Ctrl', 'Shift'], action: 'marks:delete' }
    ]
  });
}
