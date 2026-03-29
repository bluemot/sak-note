import { uiRegistry } from '../../ui-system/ModuleUIRegistry';

export function registerEditorMenuModule() {
  uiRegistry.registerModule({
    module: 'editor-menu',
    version: '1.0',
    components: [
      {
        id: 'editor-menu-trigger',
        type: 'menu_item',
        slot: 'menu.editor',
        title: 'Editor',
        module: 'editor-menu'
      }
    ],
    menus: [
      {
        id: 'editor-menu',
        slot: 'menu.editor',
        title: 'Editor',
        items: [
          { id: 'word-wrap', type: 'item', title: 'Toggle Word Wrap', action: 'editor:word_wrap' },
          { id: 'line-numbers', type: 'item', title: 'Toggle Line Numbers', action: 'editor:line_numbers' },
          { id: 'minimap', type: 'item', title: 'Toggle Minimap', action: 'editor:minimap' },
          { id: 'sep1', type: 'separator' },
          { id: 'zoom-in', type: 'item', title: 'Zoom In', action: 'editor:zoom_in', shortcut: 'Ctrl++' },
          { id: 'zoom-out', type: 'item', title: 'Zoom Out', action: 'editor:zoom_out', shortcut: 'Ctrl+-' },
          { id: 'reset-zoom', type: 'item', title: 'Reset Zoom', action: 'editor:zoom_reset', shortcut: 'Ctrl+0' },
          { id: 'sep2', type: 'separator' },
          { id: 'fold-all', type: 'item', title: 'Fold All', action: 'editor:fold_all' },
          { id: 'unfold-all', type: 'item', title: 'Unfold All', action: 'editor:unfold_all' },
          { id: 'sep3', type: 'separator' },
          { id: 'command-palette', type: 'item', title: 'Command Palette', action: 'editor:command_palette', shortcut: 'Ctrl+Shift+P' }
        ]
      }
    ],
    shortcuts: [
      { key: 'equal', modifiers: ['Ctrl'], action: 'editor:zoom_in' },
      { key: 'minus', modifiers: ['Ctrl'], action: 'editor:zoom_out' },
      { key: '0', modifiers: ['Ctrl'], action: 'editor:zoom_reset' }
    ]
  });
}
