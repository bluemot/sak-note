import React, { useState, useEffect } from 'react';
import { UIComponentDefinition, uiRegistry, UISlot } from '../ModuleUIRegistry';
import { useCondition } from '../hooks/useCondition';
import { useUIStore } from '../../store/uiStore';
import { actionRegistry } from '../actions/actionRegistry';
import './DynamicStatusBar.css';

// Icon mapping for status bar items
const iconMap: Record<string, string> = {
  'file': '📄',
  'save': '💾',
  'undo': '↩️',
  'redo': '↪️',
  'modified': '●',
  'notification': '🔔',
  'notification-empty': '🔕',
  'info': 'ℹ️',
  'success': '✅',
  'error': '❌',
  'warning': '⚠️',
  'loading': '⏳',
  'encoding': '📝',
  'cursor': '🖱️',
  'selection': '📋',
  'line': '📏',
  'column': '📐',
  'default': '•',
};

interface StatusBarItemProps {
  component: UIComponentDefinition;
}

const StatusBarItem: React.FC<StatusBarItemProps> = ({ component }) => {
  const isVisible = useCondition(component.when || 'always');

  if (!isVisible) return null;

  const handleClick = async () => {
    if (component.action) {
      try {
        await actionRegistry.execute(component.action);
      } catch (error) {
        // Error handled in actionRegistry
      }
    }
  };

  const icon = iconMap[component.icon || ''] || component.icon || iconMap['default'];

  return (
    <button
      className={`status-bar-item ${component.className || ''} ${component.action ? 'clickable' : ''}`}
      onClick={handleClick}
      title={component.tooltip || component.title}
      data-action={component.action}
      data-module={component.module}
    >
      {icon && <span className="status-item-icon">{icon}</span>}
      <span className="status-item-label">{component.title}</span>
    </button>
  );
};

export const DynamicStatusBar: React.FC = () => {
  const [currentTime, setCurrentTime] = useState(new Date());
  
  // Subscribe to UI store state
  const {
    hasFileOpen,
    currentFileName,
    canUndo,
    canRedo,
    hasChanges,
    notifications,
    isLoading,
    loadingMessage,
    hasSelection,
    selectionStart,
    selectionEnd,
  } = useUIStore();

  // Get dynamic components from statusBar slot
  const slotComponents = uiRegistry.getComponentsForSlot('statusBar.main' as UISlot);

  // Update time every second
  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  // Format time
  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    });
  };

  // Get notification count
  const notificationCount = notifications.length;
  const hasNotifications = notificationCount > 0;

  return (
    <div className="dynamic-statusbar">
      {/* Left section - File info */}
      <div className="statusbar-section statusbar-left">
        {hasFileOpen ? (
          <div className="status-file-info">
            <span className="status-file-icon">📄</span>
            <span className="status-file-name" title={currentFileName || undefined}>
              {currentFileName || 'Untitled'}
            </span>
            {hasChanges && (
              <span className="status-modified-indicator" title="Unsaved changes">
                ●
              </span>
            )}
          </div>
        ) : (
          <span className="status-no-file">No file open</span>
        )}
      </div>

      {/* Center section - Undo/Redo state and dynamic items */}
      <div className="statusbar-section statusbar-center">
        {hasFileOpen && (
          <>
            <div className="status-undo-redo">
              <span className={`status-action ${canUndo ? 'active' : 'inactive'}`}>
                ↩️ Undo
              </span>
              <span className={`status-action ${canRedo ? 'active' : 'inactive'}`}>
                ↪️ Redo
              </span>
            </div>

            <div className="status-divider" />

            {hasSelection && (
              <div className="status-selection">
                <span>Selected: {selectionEnd - selectionStart} chars</span>
              </div>
            )}

            {/* Render dynamic slot components */}
            {slotComponents.map((component) => (
              <StatusBarItem
                key={`${component.module}:${component.id}`}
                component={component}
              />
            ))}

            {/* Loading indicator */}
            {isLoading && (
              <div className="status-loading">
                <span className="loading-spinner">⏳</span>
                <span>{loadingMessage || 'Loading...'}</span>
              </div>
            )}
          </>
        )}
      </div>

      {/* Right section - Notifications and time */}
      <div className="statusbar-section statusbar-right">
        {/* Notifications */}
        <button
          className={`status-notification-btn ${hasNotifications ? 'has-notifications' : ''}`}
          title={hasNotifications ? `${notificationCount} notification(s)` : 'No notifications'}
        >
          <span className="notification-icon">
            {hasNotifications ? '🔔' : '🔕'}
          </span>
          {hasNotifications && (
            <span className="notification-count">{notificationCount}</span>
          )}
        </button>

        <div className="status-divider" />

        {/* Time */}
        <div className="status-time">
          <span>🕐</span>
          <span>{formatTime(currentTime)}</span>
        </div>
      </div>
    </div>
  );
};

export default DynamicStatusBar;
