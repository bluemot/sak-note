import { uiRegistry } from '../../ui-system/ModuleUIRegistry';

export function registerLlmModule() {
  uiRegistry.registerModule({
    module: 'llm',
    version: '1.0',
    components: [
      {
        id: 'chat',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'llm',
        title: 'AI Chat',
        icon: '🤖',
        action: 'llm:chat',
        shortcut: 'Ctrl+L',
        order: 200
      },
      {
        id: 'summarize',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'llm',
        title: 'Summarize',
        icon: '📝',
        action: 'llm:summarize',
        shortcut: 'Ctrl+Shift+L',
        order: 201,
        when: 'hasFileOpen'
      },
      {
        id: 'chat-panel',
        type: 'sidebar_panel',
        slot: 'sidebar.tabs',
        module: 'llm',
        title: '🤖 AI Chat',
        icon: '🤖',
        order: 20
      }
    ],
    menus: [
      {
        id: 'llm-menu',
        slot: 'menu.tools',
        title: 'AI',
        items: [
          { id: 'chat', type: 'item', title: 'Open Chat', action: 'llm:chat', shortcut: 'Ctrl+L' },
          { id: 'summarize', type: 'item', title: 'Summarize File', action: 'llm:summarize', shortcut: 'Ctrl+Shift+L' },
          { id: 'sep1', type: 'separator' },
          { id: 'settings', type: 'item', title: 'AI Settings...', action: 'llm:settings' }
        ]
      }
    ],
    shortcuts: [
      { key: 'l', modifiers: ['Ctrl'], action: 'llm:chat' },
      { key: 'l', modifiers: ['Ctrl', 'Shift'], action: 'llm:summarize' }
    ]
  });
}
