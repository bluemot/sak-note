import { actionRegistry } from '../../ui-system/actions/actionRegistry';
import { useUIStore } from '../../store/uiStore';
import { invoke } from '@tauri-apps/api/core';

export function registerLlmActions() {
  // Open chat
  actionRegistry.register('llm', 'chat', async (params?: {
    message?: string;
  }) => {
    try {
      useUIStore.getState().setLoading(true, 'Opening AI Chat...');

      // Open chat panel
      useUIStore.getState().addNotification({
        type: 'info',
        message: 'AI Chat opened',
        duration: 2000
      });

      // If message provided, send it
      if (params?.message) {
        const result = await invoke('llm_chat', {
          message: params.message
        });
        return result;
      }

      return { success: true };
    } catch (error) {
      console.error('[LLM Actions] Chat failed:', error);
      throw error;
    } finally {
      useUIStore.getState().setLoading(false);
    }
  });

  // Summarize current file
  actionRegistry.register('llm', 'summarize', async () => {
    try {
      const { currentFilePath } = useUIStore.getState();
      if (!currentFilePath) throw new Error('No file open');

      useUIStore.getState().setLoading(true, 'Summarizing file with AI...');

      const result = await invoke('llm_summarize', {
        file_path: currentFilePath
      });

      useUIStore.getState().addNotification({
        type: 'success',
        message: 'File summarized successfully',
        duration: 3000
      });

      return result;
    } catch (error) {
      console.error('[LLM Actions] Summarize failed:', error);
      throw error;
    } finally {
      useUIStore.getState().setLoading(false);
    }
  });

  // Open settings
  actionRegistry.register('llm', 'settings', async () => {
    try {
      // Open LLM settings dialog
      useUIStore.getState().addNotification({
        type: 'info',
        message: 'AI Settings opened',
        duration: 2000
      });
      return { success: true };
    } catch (error) {
      console.error('[LLM Actions] Settings failed:', error);
      throw error;
    }
  });

  // Configure LLM provider
  actionRegistry.register('llm', 'configure', async (params?: {
    provider?: string;
    apiKey?: string;
    model?: string;
  }) => {
    try {
      const result = await invoke('llm_configure', {
        provider: params?.provider,
        api_key: params?.apiKey,
        model: params?.model
      });

      useUIStore.getState().addNotification({
        type: 'success',
        message: 'AI provider configured',
        duration: 3000
      });

      return result;
    } catch (error) {
      console.error('[LLM Actions] Configure failed:', error);
      throw error;
    }
  });

  // Clear chat history
  actionRegistry.register('llm', 'clear_history', async () => {
    try {
      const result = await invoke('llm_clear_history');

      useUIStore.getState().addNotification({
        type: 'info',
        message: 'Chat history cleared',
        duration: 2000
      });

      return result;
    } catch (error) {
      console.error('[LLM Actions] Clear history failed:', error);
      throw error;
    }
  });
}
