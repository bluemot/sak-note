import { actionRegistry } from '../../ui-system/actions/actionRegistry';
import { useUIStore } from '../../store/uiStore';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useDialogStore } from '../../store/dialogStore';

export function registerMarksActions() {
  // Create mark at current position
  actionRegistry.register('marks', 'create', async (params?: {
    position?: number;
    label?: string;
  }) => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      // If label is not provided, open input dialog to ask for it
      let label = params?.label;
      if (!label) {
        return new Promise((resolve, reject) => {
          useDialogStore.getState().openDialog('input', {
            title: 'Create Mark',
            placeholder: 'Enter mark name...',
            onConfirm: async (value: string) => {
              try {
                const result = await invoke('marks_create', {
                  file_path: currentFilePath,
                  position: params?.position,
                  label: value
                });

                useUIStore.getState().addNotification({
                  type: 'success',
                  message: 'Mark created successfully',
                  duration: 2000
                });

                resolve(result);
              } catch (error) {
                reject(error);
              }
            }
          });
        });
      }

      const result = await invoke('marks_create', {
        file_path: currentFilePath,
        position: params?.position,
        label: label || 'Mark'
      });

      useUIStore.getState().addNotification({
        type: 'success',
        message: 'Mark created successfully',
        duration: 2000
      });

      return result;
    } catch (error) {
      console.error('[Marks Actions] Create failed:', error);
      throw error;
    }
  });

  // Delete mark at current position or by id
  actionRegistry.register('marks', 'delete', async (params?: {
    markId?: string;
  }) => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      const result = await invoke('marks_delete', {
        file_path: currentFilePath,
        mark_id: params?.markId
      });

      useUIStore.getState().addNotification({
        type: 'info',
        message: 'Mark deleted',
        duration: 2000
      });

      return result;
    } catch (error) {
      console.error('[Marks Actions] Delete failed:', error);
      throw error;
    }
  });

  // Clear all marks
  actionRegistry.register('marks', 'clear', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      const result = await invoke('marks_clear', {
        file_path: currentFilePath
      });

      useUIStore.getState().addNotification({
        type: 'info',
        message: 'All marks cleared',
        duration: 2000
      });

      return result;
    } catch (error) {
      console.error('[Marks Actions] Clear failed:', error);
      throw error;
    }
  });

  // Export marks to file
  actionRegistry.register('marks', 'export', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      const path = await save({
        defaultPath: 'marks.json',
        filters: [{ name: 'JSON', extensions: ['json'] }]
      });

      if (path) {
        const result = await invoke('marks_export', {
          file_path: currentFilePath,
          export_path: path
        });

        useUIStore.getState().addNotification({
          type: 'success',
          message: 'Marks exported successfully',
          duration: 3000
        });

        return result;
      }
    } catch (error) {
      console.error('[Marks Actions] Export failed:', error);
      throw error;
    }
  });

  // Import marks from file
  actionRegistry.register('marks', 'import', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      const selected = await open({
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }]
      });

      if (selected && typeof selected === 'string') {
        const result = await invoke('marks_import', {
          file_path: currentFilePath,
          import_path: selected
        });

        useUIStore.getState().addNotification({
          type: 'success',
          message: 'Marks imported successfully',
          duration: 3000
        });

        return result;
      }
    } catch (error) {
      console.error('[Marks Actions] Import failed:', error);
      throw error;
    }
  });
}
