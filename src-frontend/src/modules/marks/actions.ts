import { actionRegistry } from '../../ui-system/actions/actionRegistry';
import { useUIStore } from '../../store/uiStore';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useDialogStore } from '../../store/dialogStore';

// Mark color type
type MarkColor = 'red' | 'orange' | 'yellow' | 'green' | 'cyan' | 'blue' | 'purple' | 'pink' | 'gray'

const MARK_COLORS: MarkColor[] = ['red', 'orange', 'yellow', 'green', 'cyan', 'blue', 'purple', 'pink', 'gray']

// Color index for cycling
let colorIndex = 0

function getNextColor(): MarkColor {
  const color = MARK_COLORS[colorIndex % MARK_COLORS.length]
  colorIndex++
  return color
}

export function registerMarksActions() {
  // Create mark at current position
  actionRegistry.register('marks', 'create', async (params?: {
    position?: number;
    label?: string;
    color?: string;
    start?: number;
    end?: number;
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
                const result = await invoke('create_mark', {
                  req: {
                    path: currentFilePath,
                    start: params?.start ?? params?.position ?? 0,
                    end: params?.end ?? (params?.position ?? 0) + 1,
                    color: params?.color ?? 'yellow',
                    label: value,
                    note: null,
                  }
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

      const result = await invoke('create_mark', {
        req: {
          path: currentFilePath,
          start: params?.start ?? params?.position ?? 0,
          end: params?.end ?? (params?.position ?? 0) + 1,
          color: params?.color ?? 'yellow',
          label: label || 'Mark',
          note: null,
        }
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

  // Highlight selection - creates marks for all occurrences of selected text
  actionRegistry.register('marks', 'highlight_selection', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      // Use the global navigate function to get editor selection
      // The editor exposes handleCreateMark via window for cross-component communication
      const color = getNextColor()

      // Dispatch a custom event that the Editor listens to
      const event = new CustomEvent('sak-mark-highlight', { detail: { color } })
      window.dispatchEvent(event)

      useUIStore.getState().addNotification({
        type: 'info',
        message: `Highlighting selection with ${color}`,
        duration: 2000
      });
    } catch (error) {
      console.error('[Marks Actions] Highlight failed:', error);
      throw error;
    }
  });

  // Delete mark by id
  actionRegistry.register('marks', 'delete', async (params?: {
    markId?: string;
  }) => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      const result = await invoke('delete_mark', {
        req: {
          path: currentFilePath,
          id: params?.markId
        }
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

      const result = await invoke('clear_marks', {
        path: currentFilePath
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
        const result = await invoke('export_marks', {
          path: currentFilePath,
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
        const result = await invoke('import_marks', {
          path: currentFilePath,
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