import React from 'react';
import { UIComponentDefinition, uiRegistry } from '../ModuleUIRegistry';
import { useCondition } from '../hooks/useCondition';
import { actionRegistry } from '../actions/actionRegistry';
import './DynamicToolbar.css';

// 圖示對應表
const iconMap: Record<string, string> = {
  'open': '📂',
  'save': '💾',
  'save-as': '💾➕',
  'close': '❌',
  'undo': '↩️',
  'redo': '↪️',
  'cut': '✂️',
  'copy': '📋',
  'paste': '📌',
  'find': '🔍',
  'replace': '🔄',
  'search': '🔎',
  'hex': '🔢',
  'text': '📝',
  'print': '🖨️',
  'sftp': '🌐',
  'bookmark': '📑',
  'mark': '🎨',
  'settings': '⚙️',
  'plugin': '🔌',
  'default': '⬜',
};

interface ToolbarButtonProps {
  component: UIComponentDefinition;
}

const ToolbarButton: React.FC<ToolbarButtonProps> = ({ component }) => {
  // 訂閱條件狀態
  const isVisible = useCondition(component.when || 'always');
  
  if (!isVisible) return null;
  
  const handleClick = async () => {
    if (component.action) {
      try {
        await actionRegistry.execute(component.action);
      } catch (error) {
        // 錯誤已在 actionRegistry 中處理
      }
    }
  };
  
  const icon = iconMap[component.icon || ''] || component.icon || iconMap['default'];
  
  return (
    <button
      className={`toolbar-btn ${component.className || ''}`}
      title={`${component.tooltip || component.title}${component.shortcut ? ` (${component.shortcut})` : ''}`}
      onClick={handleClick}
      data-action={component.action}
      data-module={component.module}
    >
      <span className="toolbar-icon">{icon}</span>
      <span className="toolbar-label">{component.title}</span>
    </button>
  );
};

export const DynamicToolbar: React.FC = () => {
  // 讀取 toolbar.main slot 的所有組件
  const components = uiRegistry.getComponentsForSlot('toolbar.main');
  
  if (components.length === 0) {
    return (
      <div className="toolbar toolbar-empty">
        <span className="toolbar-placeholder">Toolbar (no modules registered)</span>
      </div>
    );
  }
  
  return (
    <div className="toolbar">
      {components.map((component) => (
        <ToolbarButton key={component.id} component={component} />
      ))}
    </div>
  );
};

export default DynamicToolbar;
