/**
 * NotificationContainer Component
 *
 * Toast/notification system with auto-dismiss, multiple types,
 * and action buttons support.
 */

import React, { useEffect } from 'react';
import { useUIStore, Notification } from '../../store/uiStore';
import './NotificationContainer.css';

export interface NotificationContainerProps {
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center';
  maxNotifications?: number;
}

// Individual Notification Item
interface NotificationItemProps {
  notification: Notification;
  onClose: () => void;
  onAction?: (action: string) => void;
}

const NotificationItem: React.FC<NotificationItemProps> = ({
  notification,
  onClose,
  onAction,
}) => {
  const { type, message, duration = 5000, actions } = notification;

  // Auto-dismiss timer
  useEffect(() => {
    if (duration > 0) {
      const timer = setTimeout(onClose, duration);
      return () => clearTimeout(timer);
    }
  }, [duration, onClose]);

  const getIcon = () => {
    switch (type) {
      case 'success':
        return '✓';
      case 'error':
        return '✗';
      case 'warning':
        return '⚠';
      case 'info':
      default:
        return 'ℹ';
    }
  };

  const handleActionClick = (action: string) => {
    if (onAction) {
      onAction(action);
    }
    onClose();
  };

  return (
    <div className={`notification notification--${type}`}>
      <div className="notification__icon">
        {getIcon()}
      </div>
      <div className="notification__content">
        <p className="notification__message">{message}</p>
        {actions && actions.length > 0 && (
          <div className="notification__actions">
            {actions.map((action, index) => (
              <button
                key={index}
                className="notification__action-btn"
                onClick={() => handleActionClick(action.action)}
              >
                {action.label}
              </button>
            ))}
          </div>
        )}
      </div>
      <button className="notification__close" onClick={onClose} aria-label="Close notification">
        ×
      </button>
    </div>
  );
};

export const NotificationContainer: React.FC<NotificationContainerProps> = ({
  position = 'top-right',
  maxNotifications = 5,
}) => {
  const notifications = useUIStore((state) => state.notifications);
  const removeNotification = useUIStore((state) => state.removeNotification);

  // Limit displayed notifications
  const visibleNotifications = notifications.slice(0, maxNotifications);

  const handleAction = (action: string) => {
    // Handle predefined actions
    switch (action) {
      case 'undo':
        // Dispatch undo action - could be connected to edit module
        console.log('[Notification] Undo action triggered');
        break;
      case 'retry':
        // Retry last failed operation
        console.log('[Notification] Retry action triggered');
        break;
      case 'close':
        // Close notification (already handled by onClose)
        break;
      default:
        console.log('[Notification] Custom action:', action);
    }
  };

  if (visibleNotifications.length === 0) {
    return null;
  }

  return (
    <div className={`notification-container notification-container--${position}`}>
      {visibleNotifications.map((notification) => (
        <NotificationItem
          key={notification.id}
          notification={notification}
          onClose={() => removeNotification(notification.id)}
          onAction={handleAction}
        />
      ))}
    </div>
  );
};

// Utility functions for easy notification creation
export const notify = {
  success: (message: string, options?: { duration?: number; actions?: { label: string; action: string }[] }) => {
    const { addNotification } = useUIStore.getState();
    addNotification({
      type: 'success',
      message,
      duration: options?.duration ?? 4000,
      actions: options?.actions,
    });
  },
  error: (message: string, options?: { duration?: number; actions?: { label: string; action: string }[] }) => {
    const { addNotification } = useUIStore.getState();
    addNotification({
      type: 'error',
      message,
      duration: options?.duration ?? 6000,
      actions: options?.actions ?? [{ label: 'Dismiss', action: 'close' }],
    });
  },
  warning: (message: string, options?: { duration?: number; actions?: { label: string; action: string }[] }) => {
    const { addNotification } = useUIStore.getState();
    addNotification({
      type: 'warning',
      message,
      duration: options?.duration ?? 5000,
      actions: options?.actions,
    });
  },
  info: (message: string, options?: { duration?: number; actions?: { label: string; action: string }[] }) => {
    const { addNotification } = useUIStore.getState();
    addNotification({
      type: 'info',
      message,
      duration: options?.duration ?? 3000,
      actions: options?.actions,
    });
  },
};

export default NotificationContainer;
