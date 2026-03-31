import { actionRegistry } from '../../ui-system/actions/actionRegistry';
import { useUIStore } from '../../store/uiStore';
import { invoke } from '@tauri-apps/api/core';
import { useDialogStore } from '../../store/dialogStore';

export function registerSftpActions() {
  // Connect to SFTP server
  actionRegistry.register('sftp', 'connect', async (params?: {
    host?: string;
    port?: number;
    username?: string;
    password?: string;
  }) => {
    try {
      useUIStore.getState().setLoading(true, 'Connecting to SFTP server...');

      const result = await invoke('sftp_connect', {
        host: params?.host,
        port: params?.port || 22,
        username: params?.username,
        password: params?.password
      });

      useUIStore.getState().addNotification({
        type: 'success',
        message: 'Connected to SFTP server successfully',
        duration: 3000
      });

      return result;
    } catch (error) {
      console.error('[SFTP Actions] Connect failed:', error);
      throw error;
    } finally {
      useUIStore.getState().setLoading(false);
    }
  });

  // Open remote file
  actionRegistry.register('sftp', 'open_remote', async (params?: {
    remotePath?: string;
    host?: string;
  }) => {
    try {
      useUIStore.getState().setLoading(true, 'Opening remote file...');

      const result = await invoke('sftp_open_remote', {
        remote_path: params?.remotePath,
        host: params?.host
      });

      if (result) {
        useUIStore.getState().setFileOpen(params?.remotePath || 'remote-file');
        useUIStore.getState().addNotification({
          type: 'success',
          message: 'Remote file opened successfully',
          duration: 3000
        });
      }

      return result;
    } catch (error) {
      console.error('[SFTP Actions] Open remote failed:', error);
      throw error;
    } finally {
      useUIStore.getState().setLoading(false);
    }
  });

  // Site manager
  actionRegistry.register('sftp', 'site_manager', async () => {
    console.log('[SFTP] Site Manager action triggered');
    try {
      // Open site manager dialog via dialog store
      useDialogStore.getState().openDialog('sftpSiteManager');
      return { success: true };
    } catch (error) {
      console.error('[SFTP Actions] Site Manager failed:', error);
      throw error;
    }
  });

  // Disconnect from SFTP
  actionRegistry.register('sftp', 'disconnect', async () => {
    try {
      await invoke('sftp_disconnect');
      useUIStore.getState().addNotification({
        type: 'info',
        message: 'Disconnected from SFTP server',
        duration: 2000
      });
      return { success: true };
    } catch (error) {
      console.error('[SFTP Actions] Disconnect failed:', error);
      throw error;
    }
  });
}
