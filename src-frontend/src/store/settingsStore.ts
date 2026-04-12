import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';

export interface AISettings {
  apiUrl: string;
  model: string;
  apiKey: string;
  temperature: number;
}

export interface SettingsState {
  aiSettings: AISettings;
  isApplying: boolean;

  // Actions
  updateAISettings: (settings: Partial<AISettings>) => void;
  applyAISettings: () => Promise<void>;
  loadAISettings: () => AISettings;
}

const DEFAULT_AI_SETTINGS: AISettings = {
  apiUrl: 'http://localhost:11434',
  model: 'kimi-k2.5:cloud',
  apiKey: '',
  temperature: 0.7,
};

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      aiSettings: DEFAULT_AI_SETTINGS,
      isApplying: false,

      updateAISettings: (partial) => set((state) => ({
        aiSettings: { ...state.aiSettings, ...partial }
      })),

      applyAISettings: async () => {
        set({ isApplying: true });
        try {
          const { apiUrl, model } = get().aiSettings;
          await invoke('execute_module', {
            module: 'llm',
            capability: 'set_api_url',
            input: { url: apiUrl }
          });
          await invoke('execute_module', {
            module: 'llm',
            capability: 'set_default_model',
            input: { model }
          });
        } catch (err) {
          console.error('[SettingsStore] Failed to apply AI settings:', err);
        } finally {
          set({ isApplying: false });
        }
      },

      loadAISettings: () => get().aiSettings,
    }),
    {
      name: 'sak-settings-store',
      partialize: (state) => ({ aiSettings: state.aiSettings }),
    }
  )
);

export const AVAILABLE_MODELS = [
  { value: 'kimi-k2.5:cloud', label: 'Kimi K2.5 (Cloud)' },
  { value: 'kimi-k2.5', label: 'Kimi K2.5 (Local)' },
  { value: 'llama3.2:latest', label: 'Llama 3.2' },
  { value: 'qwen2.5:latest', label: 'Qwen 2.5' },
  { value: 'deepseek-coder:latest', label: 'DeepSeek Coder' },
  { value: 'codellama:latest', label: 'Code Llama' },
  { value: 'mistral:latest', label: 'Mistral' },
  { value: 'gemma2:latest', label: 'Gemma 2' },
];