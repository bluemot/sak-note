import { uiRegistry } from '../../ui-system/ModuleUIRegistry';

export function registerSftpModule() {
  uiRegistry.registerModule({
    module: 'sftp',
    version: '1.0',
    components: [
      {
        id: 'connect',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'sftp',
        title: 'SFTP Connect',
        icon: '🌐',
        action: 'sftp:connect',
        order: 100
      },
      {
        id: 'open-remote',
        type: 'menu_item',
        slot: 'menu.file',
        module: 'sftp',
        title: 'Open Remote...',
        action: 'sftp:open_remote',
        shortcut: 'Ctrl+Shift+O'
      },
      {
        id: 'sites-panel',
        type: 'sidebar_panel',
        slot: 'sidebar.tabs',
        module: 'sftp',
        title: '🌐 SFTP Sites',
        icon: '🌐',
        order: 50
      }
    ],
    menus: [
      {
        id: 'sftp-menu',
        slot: 'menu.tools',
        title: 'SFTP',
        items: [
          { id: 'connect', type: 'item', title: 'Connect to Server...', action: 'sftp:connect', shortcut: 'Ctrl+Shift+C' },
          { id: 'open-remote', type: 'item', title: 'Open Remote File...', action: 'sftp:open_remote', shortcut: 'Ctrl+Shift+O' },
          { id: 'sep1', type: 'separator' },
          { id: 'site-manager', type: 'item', title: 'Site Manager', action: 'sftp:site_manager' },
          { id: 'disconnect', type: 'item', title: 'Disconnect', action: 'sftp:disconnect' }
        ]
      }
    ],
    shortcuts: [
      { key: 'c', modifiers: ['Ctrl', 'Shift'], action: 'sftp:connect' },
      { key: 'o', modifiers: ['Ctrl', 'Shift'], action: 'sftp:open_remote' }
    ]
  });
}
