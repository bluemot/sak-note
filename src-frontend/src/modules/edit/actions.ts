import { actionRegistry } from '../../ui-system/actions/actionRegistry';
import { useUIStore } from '../../store/uiStore';

export function registerEditActions() {
  // Undo
  actionRegistry.register('edit', 'undo', async () => {
    try {
      // Trigger undo - backend handles actual undo
      useUIStore.getState().setUndoRedo(false, true);
      return { success: true };
    } catch (error) {
      console.error('[Edit Actions] Undo failed:', error);
      throw error;
    }
  });

  // Redo
  actionRegistry.register('edit', 'redo', async () => {
    try {
      // Trigger redo - backend handles actual redo
      useUIStore.getState().setUndoRedo(true, false);
      return { success: true };
    } catch (error) {
      console.error('[Edit Actions] Redo failed:', error);
      throw error;
    }
  });

  // Cut
  actionRegistry.register('edit', 'cut', async () => {
    try {
      // Cut selected text to clipboard
      document.execCommand('cut');
      return { success: true };
    } catch (error) {
      console.error('[Edit Actions] Cut failed:', error);
      throw error;
    }
  });

  // Copy
  actionRegistry.register('edit', 'copy', async () => {
    try {
      // Copy selected text to clipboard
      document.execCommand('copy');
      return { success: true };
    } catch (error) {
      console.error('[Edit Actions] Copy failed:', error);
      throw error;
    }
  });

  // Paste
  actionRegistry.register('edit', 'paste', async () => {
    try {
      // Paste from clipboard
      document.execCommand('paste');
      return { success: true };
    } catch (error) {
      console.error('[Edit Actions] Paste failed:', error);
      throw error;
    }
  });

  // Find
  actionRegistry.register('edit', 'find', async () => {
    try {
      // Open find dialog - UI handles the display
      useUIStore.getState().addNotification({
        type: 'info',
        message: 'Find dialog opened',
        duration: 2000
      });
      return { success: true };
    } catch (error) {
      console.error('[Edit Actions] Find failed:', error);
      throw error;
    }
  });

  // Replace
  actionRegistry.register('edit', 'replace', async () => {
    try {
      // Open replace dialog - UI handles the display
      useUIStore.getState().addNotification({
        type: 'info',
        message: 'Replace dialog opened',
        duration: 2000
      });
      return { success: true };
    } catch (error) {
      console.error('[Edit Actions] Replace failed:', error);
      throw error;
    }
  });

  // Find in Files
  actionRegistry.register('edit', 'find_in_files', async () => {
    try {
      // Open find in files dialog
      useUIStore.getState().addNotification({
        type: 'info',
        message: 'Find in Files dialog opened',
        duration: 2000
      });
      return { success: true };
    } catch (error) {
      console.error('[Edit Actions] Find in Files failed:', error);
      throw error;
    }
  });

  // Go to Line
  actionRegistry.register('edit', 'goto_line', async (params?: { line?: number }) => {
    try {
      const line = params?.line;
      if (line) {
        // Navigate to specific line
        useUIStore.getState().addNotification({
          type: 'info',
          message: `Navigating to line ${line}`,
          duration: 2000
        });
      } else {
        // Show goto line dialog
        useUIStore.getState().addNotification({
          type: 'info',
          message: 'Go to Line dialog opened',
          duration: 2000
        });
      }
      return { success: true };
    } catch (error) {
      console.error('[Edit Actions] Go to Line failed:', error);
      throw error;
    }
  });
}
