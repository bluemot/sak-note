import { create } from 'zustand';

// Dialog state interface
export interface DialogState {
  // Active dialog type
  activeDialog: string | null;
  // Props for the active dialog
  dialogProps: any;
  
  // Actions
  openDialog: (dialogName: string, props?: any) => void;
  closeDialog: () => void;
}

// Global dialog store
export const useDialogStore = create<DialogState>((set) => ({
  // Initial state
  activeDialog: null,
  dialogProps: null,
  
  // Open a dialog with optional props
  openDialog: (dialogName, props = null) => {
    console.log('[DialogStore] Opening dialog:', dialogName, props);
    set({
      activeDialog: dialogName,
      dialogProps: props,
    });
  },
  
  // Close the current dialog
  closeDialog: () => set({
    activeDialog: null,
    dialogProps: null,
  }),
}));

// Dialog names for type safety
export const DIALOGS = {
  SFTP_SITE_MANAGER: 'sftpSiteManager',
  AI_SETTINGS: 'aiSettings',
  COMMAND_PALETTE: 'commandPalette',
  INPUT: 'input',
} as const;

export type DialogName = typeof DIALOGS[keyof typeof DIALOGS];
