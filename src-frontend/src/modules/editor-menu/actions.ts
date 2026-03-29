import { actionRegistry } from '../../ui-system/actions/actionRegistry';
import { useDialogStore } from '../../store/dialogStore';

export function registerEditorMenuActions() {
  // Toggle Word Wrap
  actionRegistry.register('editor', 'word_wrap', async () => {
    console.log('[Editor Menu] Toggle word wrap');
    // This would interface with Monaco Editor settings
    // For now, just log the action
  });

  // Toggle Line Numbers
  actionRegistry.register('editor', 'line_numbers', async () => {
    console.log('[Editor Menu] Toggle line numbers');
  });

  // Toggle Minimap
  actionRegistry.register('editor', 'minimap', async () => {
    console.log('[Editor Menu] Toggle minimap');
  });

  // Zoom In
  actionRegistry.register('editor', 'zoom_in', async () => {
    console.log('[Editor Menu] Zoom in');
  });

  // Zoom Out
  actionRegistry.register('editor', 'zoom_out', async () => {
    console.log('[Editor Menu] Zoom out');
  });

  // Reset Zoom
  actionRegistry.register('editor', 'zoom_reset', async () => {
    console.log('[Editor Menu] Reset zoom');
  });

  // Fold All
  actionRegistry.register('editor', 'fold_all', async () => {
    console.log('[Editor Menu] Fold all');
  });

  // Unfold All
  actionRegistry.register('editor', 'unfold_all', async () => {
    console.log('[Editor Menu] Unfold all');
  });

  // Command Palette
  actionRegistry.register('editor', 'command_palette', async () => {
    try {
      // Open command palette via dialog store
      useDialogStore.getState().openDialog('commandPalette');
      return { success: true };
    } catch (error) {
      console.error('[Editor Menu] Command Palette failed:', error);
      throw error;
    }
  });
}
