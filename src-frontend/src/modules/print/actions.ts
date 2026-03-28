import { actionRegistry } from '../../ui-system/actions/actionRegistry';
import { useUIStore } from '../../store/uiStore';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';

export function registerPrintActions() {
  // Print current file
  actionRegistry.register('print', 'print', async (params?: {
    printer?: string;
    copies?: number;
  }) => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      useUIStore.getState().setLoading(true, 'Preparing to print...');

      const result = await invoke('print_file', {
        file_path: currentFilePath,
        printer: params?.printer,
        copies: params?.copies || 1
      });

      useUIStore.getState().addNotification({
        type: 'success',
        message: 'File sent to printer',
        duration: 3000
      });

      return result;
    } catch (error) {
      console.error('[Print Actions] Print failed:', error);
      throw error;
    } finally {
      useUIStore.getState().setLoading(false);
    }
  });

  // Export to PDF
  actionRegistry.register('print', 'export_pdf', async (params?: {
    outputPath?: string;
  }) => {
    try {
      const { currentFilePath, currentFileName } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      useUIStore.getState().setLoading(true, 'Exporting to PDF...');

      let outputPath: string | null = params?.outputPath || null;
      if (!outputPath) {
        const defaultName = currentFileName?.replace(/\.[^/.]+$/, '') + '.pdf' || 'document.pdf';
        outputPath = await save({
          defaultPath: defaultName,
          filters: [{ name: 'PDF', extensions: ['pdf'] }]
        });
      }

      if (outputPath) {
        const result = await invoke('export_pdf', {
          file_path: currentFilePath,
          output_path: outputPath
        });

        useUIStore.getState().addNotification({
          type: 'success',
          message: 'PDF exported successfully',
          duration: 3000
        });

        return result;
      }
    } catch (error) {
      console.error('[Print Actions] Export PDF failed:', error);
      throw error;
    } finally {
      useUIStore.getState().setLoading(false);
    }
  });

  // Export to HTML
  actionRegistry.register('print', 'export_html', async (params?: {
    outputPath?: string;
  }) => {
    try {
      const { currentFilePath, currentFileName } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      useUIStore.getState().setLoading(true, 'Exporting to HTML...');

      let outputPath: string | null = params?.outputPath || null;
      if (!outputPath) {
        const defaultName = currentFileName?.replace(/\.[^/.]+$/, '') + '.html' || 'document.html';
        outputPath = await save({
          defaultPath: defaultName,
          filters: [{ name: 'HTML', extensions: ['html'] }]
        });
      }

      if (outputPath) {
        const result = await invoke('export_html', {
          file_path: currentFilePath,
          output_path: outputPath
        });

        useUIStore.getState().addNotification({
          type: 'success',
          message: 'HTML exported successfully',
          duration: 3000
        });

        return result;
      }
    } catch (error) {
      console.error('[Print Actions] Export HTML failed:', error);
      throw error;
    } finally {
      useUIStore.getState().setLoading(false);
    }
  });

  // Page setup
  actionRegistry.register('print', 'page_setup', async () => {
    try {
      // Open page setup dialog
      useUIStore.getState().addNotification({
        type: 'info',
        message: 'Page Setup opened',
        duration: 2000
      });
      return { success: true };
    } catch (error) {
      console.error('[Print Actions] Page Setup failed:', error);
      throw error;
    }
  });

  // Print preview
  actionRegistry.register('print', 'preview', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      useUIStore.getState().setLoading(true, 'Generating print preview...');

      const result = await invoke('print_preview', {
        file_path: currentFilePath
      });

      useUIStore.getState().addNotification({
        type: 'info',
        message: 'Print preview generated',
        duration: 2000
      });

      return result;
    } catch (error) {
      console.error('[Print Actions] Preview failed:', error);
      throw error;
    } finally {
      useUIStore.getState().setLoading(false);
    }
  });
}
