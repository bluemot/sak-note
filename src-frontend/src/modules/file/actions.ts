import { actionRegistry } from '../../ui-system/actions/actionRegistry';
import { useUIStore } from '../../store/uiStore';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

export function registerFileActions() {
  // Open file
  actionRegistry.register('file', 'open', async () => {
    try {
      const selected = await open({ multiple: false });
      if (selected && typeof selected === 'string') {
        const fileInfo = await invoke('open_file', { path: selected });
        useUIStore.getState().setFileOpen(selected);
        return fileInfo;
      }
    } catch (error) {
      console.error('[File Actions] Open failed:', error);
      throw error;
    }
  });

  // New file
  actionRegistry.register('file', 'new', async () => {
    try {
      useUIStore.getState().setFileOpen(null);
      return { success: true };
    } catch (error) {
      console.error('[File Actions] New file failed:', error);
      throw error;
    }
  });

  // Save file
  actionRegistry.register('file', 'save', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');
      await invoke('save_file', { path: currentFilePath });
      useUIStore.getState().setHasChanges(false);
    } catch (error) {
      console.error('[File Actions] Save failed:', error);
      throw error;
    }
  });

  // Save As
  actionRegistry.register('file', 'save_as', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      const path = await save({
        defaultPath: currentFilePath || undefined
      });
      if (path) {
        await invoke('save_as', {
          source_path: currentFilePath,
          target_path: path
        });
        useUIStore.getState().setFileOpen(path);
      }
    } catch (error) {
      console.error('[File Actions] Save As failed:', error);
      throw error;
    }
  });

  // Close file
  actionRegistry.register('file', 'close', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (currentFilePath) {
        await invoke('close_file', { path: currentFilePath });
      }
      useUIStore.getState().setFileOpen(null);
    } catch (error) {
      console.error('[File Actions] Close failed:', error);
      throw error;
    }
  });

  // Open recent
  actionRegistry.register('file', 'recent', async () => {
    try {
      // Open recent files menu - handled by UI
      return { success: true };
    } catch (error) {
      console.error('[File Actions] Recent failed:', error);
      throw error;
    }
  });
}
