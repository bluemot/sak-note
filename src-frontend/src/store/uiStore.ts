import { create } from 'zustand';
import { persist } from 'zustand/middleware';

// Notification 類型
export interface Notification {
  id: string;
  type: 'success' | 'error' | 'info' | 'warning';
  message: string;
  duration?: number;
}

// UI 狀態介面
export interface UIState {
  // 檔案狀態
  hasFileOpen: boolean;
  currentFilePath: string | null;
  currentFileName: string | null;
  canUndo: boolean;
  canRedo: boolean;
  hasChanges: boolean;
  
  // 選擇狀態
  hasSelection: boolean;
  selectionStart: number;
  selectionEnd: number;
  
  // 載入狀態
  isLoading: boolean;
  loadingMessage: string;
  
  // 通知
  notifications: Notification[];
  
  // Actions
  setFileOpen: (path: string | null, name?: string | null) => void;
  setUndoRedo: (canUndo: boolean, canRedo: boolean) => void;
  setHasChanges: (hasChanges: boolean) => void;
  setSelection: (hasSelection: boolean, start?: number, end?: number) => void;
  setLoading: (isLoading: boolean, message?: string) => void;
  addNotification: (notification: Omit<Notification, 'id'>) => void;
  removeNotification: (id: string) => void;
  clearNotifications: () => void;
}

// 建立 Store
export const useUIStore = create<UIState>()(
  persist(
    (set) => ({
      // 初始狀態
      hasFileOpen: false,
      currentFilePath: null,
      currentFileName: null,
      canUndo: false,
      canRedo: false,
      hasChanges: false,
      hasSelection: false,
      selectionStart: 0,
      selectionEnd: 0,
      isLoading: false,
      loadingMessage: '',
      notifications: [],
      
      // Actions
      setFileOpen: (path, name) => set({
        hasFileOpen: !!path,
        currentFilePath: path,
        currentFileName: name || (path ? path.split('/').pop() || null : null),
        hasChanges: false,
      }),
      
      setUndoRedo: (canUndo, canRedo) => set({
        canUndo,
        canRedo,
      }),
      
      setHasChanges: (hasChanges) => set({
        hasChanges,
      }),
      
      setSelection: (hasSelection, start, end) => set({
        hasSelection,
        selectionStart: start || 0,
        selectionEnd: end || 0,
      }),
      
      setLoading: (isLoading, message = '') => set({
        isLoading,
        loadingMessage: message,
      }),
      
      addNotification: (notification) => set((state) => ({
        notifications: [
          ...state.notifications,
          {
            ...notification,
            id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
          },
        ],
      })),
      
      removeNotification: (id) => set((state) => ({
        notifications: state.notifications.filter((n) => n.id !== id),
      })),
      
      clearNotifications: () => set({
        notifications: [],
      }),
    }),
    {
      name: 'sak-editor-ui-store',
      // 只持續化部分狀態
      partialize: (state) => ({
        // 檔案相關不持續化（重啟時應該從頭開始）
        // 只持續化設定
      }),
    }
  )
);

// 條件評估函數對應表
export const conditionEvaluators: Record<string, (state: UIState) => boolean> = {
  hasFileOpen: (state) => state.hasFileOpen,
  canUndo: (state) => state.canUndo,
  canRedo: (state) => state.canRedo,
  hasSelection: (state) => state.hasSelection,
  hasChanges: (state) => state.hasChanges,
  noFileOpen: (state) => !state.hasFileOpen,
  always: () => true,
};

// 評估條件
export function evaluateCondition(condition: string, state: UIState): boolean {
  const evaluator = conditionEvaluators[condition];
  return evaluator ? evaluator(state) : true;
}
