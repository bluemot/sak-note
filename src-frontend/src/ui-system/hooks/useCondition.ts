import { useUIStore, evaluateCondition, UIState } from '../../store/uiStore';
import { useEffect, useState } from 'react';

/**
 * React Hook: 訂閱條件狀態變化
 * 
 * 使用 Zustand selector 訂閱特定條件，當相關狀態變化時自動響應
 */
export function useCondition(condition: string): boolean {
  // 使用 Zustand selector 訂閱相關狀態
  return useUIStore((state: UIState) => {
    return evaluateCondition(condition, state);
  });
}

/**
 * React Hook: 訂閱多個條件
 */
export function useConditions(conditions: string[]): boolean[] {
  return useUIStore((state: UIState) => {
    return conditions.map(cond => evaluateCondition(cond, state));
  });
}

/**
 * React Hook: 至少一個條件為真
 */
export function useAnyCondition(conditions: string[]): boolean {
  return useUIStore((state: UIState) => {
    return conditions.some(cond => evaluateCondition(cond, state));
  });
}

/**
 * React Hook: 所有條件都為真
 */
export function useAllConditions(conditions: string[]): boolean {
  return useUIStore((state: UIState) => {
    return conditions.every(cond => evaluateCondition(cond, state));
  });
}
