import React, { useState, Suspense } from 'react';
import { UIComponentDefinition, uiRegistry } from '../ModuleUIRegistry';
import { useCondition } from '../hooks/useCondition';
import './DynamicSidebar.css';

// Icon mapping for sidebar tabs
const iconMap: Record<string, string> = {
  'bookmark': '📑',
  'bookmarks': '📑',
  'globe': '🌐',
  'brain': '🧠',
  'search': '🔍',
  'chat': '💬',
  'marks': '🎨',
  'info': 'ℹ️',
  'settings': '⚙️',
  'folder': '📁',
  'file': '📄',
  'terminal': '💻',
  'debug': '🐛',
  'extensions': '🔌',
  'default': '📋',
};

interface SidebarTabPanelProps {
  component: UIComponentDefinition;
  isActive: boolean;
}

const SidebarTabPanel: React.FC<SidebarTabPanelProps> = ({ component, isActive }) => {
  if (!isActive) return null;

  // If component has a component reference, try to render it
  if (component.component) {
    // For built-in components, we can map them to actual imports
    // This is a simplified version - in production, you'd use dynamic imports
    return (
      <div className="dynamic-sidebar-panel-content" data-component={component.component}>
        <div className="panel-placeholder">
          <span className="panel-icon">{iconMap[component.icon || ''] || iconMap['default']}</span>
          <span className="panel-title">{component.title}</span>
          <span className="panel-module">from {component.module}</span>
        </div>
      </div>
    );
  }

  // Default panel content
  return (
    <div className="dynamic-sidebar-panel-content">
      <div className="panel-placeholder">
        <span className="panel-icon">{iconMap[component.icon || ''] || iconMap['default']}</span>
        <span className="panel-title">{component.title}</span>
        <span className="panel-module">Module: {component.module}</span>
        {component.action && (
          <span className="panel-action">Action: {component.action}</span>
        )}
      </div>
    </div>
  );
};

interface SidebarTabButtonProps {
  component: UIComponentDefinition;
  isActive: boolean;
  onClick: () => void;
}

const SidebarTabButton: React.FC<SidebarTabButtonProps> = ({ component, isActive, onClick }) => {
  // Subscribe to condition state
  const isVisible = useCondition(component.when || 'always');

  if (!isVisible) return null;

  const icon = iconMap[component.icon || ''] || component.icon || iconMap['default'];

  return (
    <button
      className={`dynamic-sidebar-tab ${isActive ? 'active' : ''} ${component.className || ''}`}
      onClick={onClick}
      title={component.tooltip || component.title}
      data-component-id={component.id}
      data-module={component.module}
    >
      <span className="tab-icon">{icon}</span>
      <span className="tab-label">{component.title}</span>
    </button>
  );
};

export interface DynamicSidebarProps {
  currentFilePath?: string | null;
  currentFileName?: string | null;
}

export const DynamicSidebar: React.FC<DynamicSidebarProps> = ({ currentFilePath, currentFileName }) => {
  // Read components from sidebar.tabs slot
  const components = uiRegistry.getComponentsForSlot('sidebar.tabs');
  const [activeTabId, setActiveTabId] = useState<string | null>(
    components.length > 0 ? components[0].id : null
  );

  // Filter visible components
  const visibleComponents = components.filter(c => c.visible !== false);

  if (visibleComponents.length === 0) {
    return (
      <div className="dynamic-sidebar dynamic-sidebar-empty">
        <div className="dynamic-sidebar-tabs">
          <span className="sidebar-placeholder">No sidebar panels registered</span>
        </div>
        <div className="dynamic-sidebar-content">
          <div className="empty-panel">
            <span className="empty-icon">📂</span>
            <span className="empty-text">No modules loaded</span>
          </div>
        </div>
      </div>
    );
  }

  // Ensure active tab is valid
  const activeComponent = visibleComponents.find(c => c.id === activeTabId) || visibleComponents[0];
  const effectiveActiveId = activeComponent.id;

  const handleTabClick = (componentId: string) => {
    setActiveTabId(componentId);
  };

  return (
    <div className="dynamic-sidebar">
      <div className="dynamic-sidebar-tabs">
        {visibleComponents.map((component) => (
          <SidebarTabButton
            key={`${component.module}:${component.id}`}
            component={component}
            isActive={effectiveActiveId === component.id}
            onClick={() => handleTabClick(component.id)}
          />
        ))}
      </div>

      <div className="dynamic-sidebar-content">
        <Suspense fallback={
          <div className="sidebar-loading">
            <span className="loading-spinner">⏳</span>
            <span>Loading...</span>
          </div>
        }>
          {/* File info panel - always available as first tab */}
          {effectiveActiveId === 'info' && (
            <div className="info-panel">
              {currentFilePath ? (
                <>
                  <div className="info-item">
                    <span className="label">File:</span>
                    <span className="value" title={currentFilePath}>
                      {currentFileName || currentFilePath.split('/').pop() || currentFilePath.split('\\').pop() || currentFilePath}
                    </span>
                  </div>
                </>
              ) : (
                <p className="no-file">No file open</p>
              )}
            </div>
          )}

          {/* Render dynamic panels */}
          {visibleComponents.map((component) => (
            <SidebarTabPanel
              key={`${component.module}:${component.id}-panel`}
              component={component}
              isActive={effectiveActiveId === component.id}
            />
          ))}
        </Suspense>
      </div>
    </div>
  );
};

export default DynamicSidebar;
