import { actionRegistry } from '../../ui-system/actions/actionRegistry';
import { useUIStore } from '../../store/uiStore';
import { invoke } from '@tauri-apps/api/core';

export function registerBookmarkActions() {
  // Toggle bookmark at current line
  actionRegistry.register('bookmark', 'toggle', async (params?: {
    line?: number;
  }) => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      const result = await invoke('bookmark_toggle', {
        file_path: currentFilePath,
        line: params?.line
      });

      useUIStore.getState().addNotification({
        type: 'success',
        message: 'Bookmark toggled',
        duration: 2000
      });

      return result;
    } catch (error) {
      console.error('[Bookmark Actions] Toggle failed:', error);
      throw error;
    }
  });

  // Navigate to next bookmark
  actionRegistry.register('bookmark', 'next', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      const result = await invoke('bookmark_next', {
        file_path: currentFilePath
      });

      return result;
    } catch (error) {
      console.error('[Bookmark Actions] Next failed:', error);
      throw error;
    }
  });

  // Navigate to previous bookmark
  actionRegistry.register('bookmark', 'prev', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      const result = await invoke('bookmark_prev', {
        file_path: currentFilePath
      });

      return result;
    } catch (error) {
      console.error('[Bookmark Actions] Previous failed:', error);
      throw error;
    }
  });

  // Clear all bookmarks
  actionRegistry.register('bookmark', 'clear', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      const result = await invoke('bookmark_clear', {
        file_path: currentFilePath
      });

      useUIStore.getState().addNotification({
        type: 'info',
        message: 'All bookmarks cleared',
        duration: 2000
      });

      return result;
    } catch (error) {
      console.error('[Bookmark Actions] Clear failed:', error);
      throw error;
    }
  });
}
