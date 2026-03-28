import { uiRegistry } from '../../ui-system/ModuleUIRegistry';

export function registerBookmarkModule() {
  uiRegistry.registerModule({
    module: 'bookmark',
    version: '1.0',
    components: [
      {
        id: 'toggle',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'bookmark',
        title: 'Toggle Bookmark',
        icon: '🔖',
        action: 'bookmark:toggle',
        shortcut: 'Ctrl+F2',
        order: 50,
        when: 'hasFileOpen'
      },
      {
        id: 'next',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'bookmark',
        title: 'Next Bookmark',
        icon: '⬇️',
        action: 'bookmark:next',
        shortcut: 'F2',
        order: 51,
        when: 'hasFileOpen'
      },
      {
        id: 'prev',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'bookmark',
        title: 'Prev Bookmark',
        icon: '⬆️',
        action: 'bookmark:prev',
        shortcut: 'Shift+F2',
        order: 52,
        when: 'hasFileOpen'
      },
      {
        id: 'panel',
        type: 'sidebar_panel',
        slot: 'sidebar.tabs',
        module: 'bookmark',
        title: '📑 Bookmarks',
        icon: '🔖',
        order: 40
      }
    ],
    menus: [
      {
        id: 'bookmark-menu',
        slot: 'menu.tools',
        title: 'Bookmarks',
        items: [
          { id: 'toggle', type: 'item', title: 'Toggle Bookmark', action: 'bookmark:toggle', shortcut: 'Ctrl+F2' },
          { id: 'next', type: 'item', title: 'Next Bookmark', action: 'bookmark:next', shortcut: 'F2' },
          { id: 'prev', type: 'item', title: 'Previous Bookmark', action: 'bookmark:prev', shortcut: 'Shift+F2' },
          { id: 'sep1', type: 'separator' },
          { id: 'clear', type: 'item', title: 'Clear All Bookmarks', action: 'bookmark:clear' }
        ]
      }
    ],
    shortcuts: [
      { key: 'F2', modifiers: [], action: 'bookmark:next' },
      { key: 'F2', modifiers: ['Shift'], action: 'bookmark:prev' },
      { key: 'F2', modifiers: ['Ctrl'], action: 'bookmark:toggle' }
    ]
  });
}
