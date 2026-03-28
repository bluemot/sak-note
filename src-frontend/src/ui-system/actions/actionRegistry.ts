import { useUIStore } from '../../store/uiStore';

// Action Handler 類型
export type ActionHandler = (params?: any) => Promise<any>;

// Action Registry 類別
class ActionRegistry {
  private handlers: Map<string, ActionHandler> = new Map();
  
  /**
   * 註冊 Action Handler
   */
  register(moduleId: string, actionId: string, handler: ActionHandler): void {
    const fullActionId = `${moduleId}:${actionId}`;
    this.handlers.set(fullActionId, handler);
    console.log(`[ActionRegistry] Registered: ${fullActionId}`);
  }
  
  /**
   * 批次註冊 Actions
   */
  registerMany(moduleId: string, actions: Record<string, ActionHandler>): void {
    Object.entries(actions).forEach(([actionId, handler]) => {
      this.register(moduleId, actionId, handler);
    });
  }
  
  /**
   * 執行 Action
   */
  async execute(actionId: string, params?: any): Promise<any> {
    const handler = this.handlers.get(actionId);
    
    if (!handler) {
      const error = `Action "${actionId}" not found`;
      console.error(`[ActionRegistry] ${error}`);
      
      // 顯示錯誤通知
      useUIStore.getState().addNotification({
        type: 'error',
        message: error,
        duration: 5000,
      });
      
      throw new Error(error);
    }
    
    try {
      // 開始載入
      useUIStore.getState().setLoading(true, `Executing ${actionId}...`);
      
      // 執行 handler
      const result = await handler(params);
      
      // 成功通知
      useUIStore.getState().addNotification({
        type: 'success',
        message: `${actionId} completed successfully`,
        duration: 2000,
      });
      
      return result;
      
    } catch (error) {
      // 錯誤處理
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      
      console.error(`[ActionRegistry] Error executing ${actionId}:`, error);
      
      // 錯誤通知
      useUIStore.getState().addNotification({
        type: 'error',
        message: `${actionId} failed: ${errorMessage}`,
        duration: 5000,
      });
      
      throw error;
      
    } finally {
      // 結束載入
      useUIStore.getState().setLoading(false);
    }
  }
  
  /**
   * 卸載 Module 的所有 Actions
   */
  unregisterAll(moduleId: string): void {
    const prefix = `${moduleId}:`;
    for (const [key] of this.handlers) {
      if (key.startsWith(prefix)) {
        this.handlers.delete(key);
      }
    }
    console.log(`[ActionRegistry] Unregistered all actions for module: ${moduleId}`);
  }
  
  /**
   * 檢查 Action 是否存在
   */
  has(actionId: string): boolean {
    return this.handlers.has(actionId);
  }
  
  /**
   * 獲取所有已註冊的 Actions
   */
  getAllActions(): string[] {
    return Array.from(this.handlers.keys());
  }
}

// 全局實例
export const actionRegistry = new ActionRegistry();

// React Hook
export function useActionRegistry() {
  return {
    execute: actionRegistry.execute.bind(actionRegistry),
    has: actionRegistry.has.bind(actionRegistry),
  };
}
