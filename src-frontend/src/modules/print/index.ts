import { uiRegistry } from '../../ui-system/ModuleUIRegistry';

export function registerPrintModule() {
  uiRegistry.registerModule({
    module: 'print',
    version: '1.0',
    components: [
      {
        id: 'print',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'print',
        title: 'Print',
        icon: '🖨️',
        action: 'print:print',
        shortcut: 'Ctrl+P',
        order: 300,
        when: 'hasFileOpen'
      },
      {
        id: 'export-pdf',
        type: 'toolbar_button',
        slot: 'toolbar.main',
        module: 'print',
        title: 'Export PDF',
        icon: '📄',
        action: 'print:export_pdf',
        shortcut: 'Ctrl+Shift+E',
        order: 301,
        when: 'hasFileOpen'
      }
    ],
    menus: [
      {
        id: 'print-menu',
        slot: 'menu.file',
        title: 'Print',
        items: [
          { id: 'print', type: 'item', title: 'Print...', action: 'print:print', shortcut: 'Ctrl+P' },
          { id: 'sep1', type: 'separator' },
          { id: 'export-pdf', type: 'item', title: 'Export as PDF...', action: 'print:export_pdf', shortcut: 'Ctrl+Shift+E' },
          { id: 'export-html', type: 'item', title: 'Export as HTML...', action: 'print:export_html' }
        ]
      }
    ],
    shortcuts: [
      { key: 'p', modifiers: ['Ctrl'], action: 'print:print' },
      { key: 'e', modifiers: ['Ctrl', 'Shift'], action: 'print:export_pdf' }
    ]
  });
}
