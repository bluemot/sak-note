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
        id: 'site-manager',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'sftp',
        title: 'SFTP Sites',
        icon: '📂',
        action: 'sftp:site_manager',
        order: 101
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
      },
      // SFTP Submenu in Tools menu
      {
        id: 'sftp-submenu',
        type: 'menu_submenu',
        slot: 'menu.tools',
        module: 'sftp',
        title: 'SFTP',
        icon: '🌐',
        order: 100
      },
      // SFTP submenu items - using group to link to parent submenu
      {
        id: 'sftp-connect',
        type: 'menu_item',
        slot: 'menu.tools',
        module: 'sftp',
        title: 'Connect to Server...',
        action: 'sftp:connect',
        shortcut: 'Ctrl+Shift+C',
        group: 'sftp-submenu',
        order: 1
      },
      {
        id: 'sftp-open-remote',
        type: 'menu_item',
        slot: 'menu.tools',
        module: 'sftp',
        title: 'Open Remote File...',
        action: 'sftp:open_remote',
        shortcut: 'Ctrl+Shift+O',
        group: 'sftp-submenu',
        order: 2
      },
      {
        id: 'sftp-sep1',
        type: 'menu_item',
        slot: 'menu.tools',
        module: 'sftp',
        title: '---',
        group: 'sftp-submenu',
        order: 3
      },
      {
        id: 'sftp-site-manager',
        type: 'menu_item',
        slot: 'menu.tools',
        module: 'sftp',
        title: 'Site Manager',
        action: 'sftp:site_manager',
        group: 'sftp-submenu',
        order: 4
      },
      {
        id: 'sftp-disconnect',
        type: 'menu_item',
        slot: 'menu.tools',
        module: 'sftp',
        title: 'Disconnect',
        action: 'sftp:disconnect',
        group: 'sftp-submenu',
        order: 5
      }
    ],
    shortcuts: [
      { key: 'c', modifiers: ['Ctrl', 'Shift'], action: 'sftp:connect' },
      { key: 'o', modifiers: ['Ctrl', 'Shift'], action: 'sftp:open_remote' }
    ]
  });
}
