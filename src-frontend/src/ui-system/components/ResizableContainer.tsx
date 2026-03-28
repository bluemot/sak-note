/**
 * ResizableContainer Component
 *
 * A container that allows resizing via drag handle.
 * Supports horizontal and vertical resizing with min/max constraints.
 * Persists size to localStorage.
 */

import React, { useState, useRef, useEffect, useCallback } from 'react';
import './ResizableContainer.css';

export interface ResizableContainerProps {
  direction: 'horizontal' | 'vertical';
  minSize?: number;
  maxSize?: number;
  defaultSize?: number;
  storageKey?: string; // localStorage key for persistence
  children: React.ReactNode;
}

export const ResizableContainer: React.FC<ResizableContainerProps> = ({
  direction,
  minSize = 100,
  maxSize = 600,
  defaultSize = 250,
  storageKey,
  children,
}) => {
  // Get initial size from localStorage or default
  const getInitialSize = useCallback(() => {
    if (storageKey) {
      const stored = localStorage.getItem(storageKey);
      if (stored) {
        const size = parseInt(stored, 10);
        if (!isNaN(size) && size >= minSize && size <= maxSize) {
          return size;
        }
      }
    }
    return defaultSize;
  }, [storageKey, defaultSize, minSize, maxSize]);

  const [size, setSize] = useState(getInitialSize);
  const [isResizing, setIsResizing] = useState(false);
  const startPosRef = useRef(0);
  const startSizeRef = useRef(0);
  const containerRef = useRef<HTMLDivElement>(null);

  // Handle mouse down on resize handle
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();

    setIsResizing(true);
    startPosRef.current = direction === 'horizontal' ? e.clientX : e.clientY;
    startSizeRef.current = size;

    // Add resizing class to body for cursor
    document.body.classList.add('resizing');
    if (direction === 'horizontal') {
      document.body.classList.add('resizing-horizontal');
    } else {
      document.body.classList.add('resizing-vertical');
    }
  }, [direction, size]);

  // Handle mouse move during resize
  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isResizing) return;

    const currentPos = direction === 'horizontal' ? e.clientX : e.clientY;
    const delta = currentPos - startPosRef.current;

    let newSize = startSizeRef.current;
    if (direction === 'horizontal') {
      // For horizontal, if it's the left panel (sidebar), we add delta
      // The resize handle is on the right edge of the container
      newSize = startSizeRef.current + delta;
    } else {
      // For vertical, add delta
      newSize = startSizeRef.current + delta;
    }

    // Clamp to min/max
    newSize = Math.max(minSize, Math.min(maxSize, newSize));
    setSize(newSize);
  }, [isResizing, direction, minSize, maxSize]);

  // Handle mouse up to end resize
  const handleMouseUp = useCallback(() => {
    if (isResizing) {
      setIsResizing(false);

      // Remove resizing classes
      document.body.classList.remove('resizing');
      document.body.classList.remove('resizing-horizontal');
      document.body.classList.remove('resizing-vertical');

      // Persist to localStorage
      if (storageKey) {
        localStorage.setItem(storageKey, size.toString());
      }
    }
  }, [isResizing, storageKey, size]);

  // Attach global mouse events during resize
  useEffect(() => {
    if (isResizing) {
      document.addEventListener('mousemove', handleMouseMove, { passive: true });
      document.addEventListener('mouseup', handleMouseUp);

      return () => {
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [isResizing, handleMouseMove, handleMouseUp]);

  // Handle window resize - ensure we don't exceed bounds
  useEffect(() => {
    const handleWindowResize = () => {
      const container = containerRef.current;
      if (!container) return;

      const parentRect = container.parentElement?.getBoundingClientRect();
      if (parentRect) {
        const maxAllowedSize = direction === 'horizontal'
          ? parentRect.width * 0.8
          : parentRect.height * 0.8;

        if (size > maxAllowedSize) {
          const newSize = Math.max(minSize, Math.min(maxSize, maxAllowedSize));
          setSize(newSize);
          if (storageKey) {
            localStorage.setItem(storageKey, newSize.toString());
          }
        }
      }
    };

    window.addEventListener('resize', handleWindowResize, { passive: true });
    return () => window.removeEventListener('resize', handleWindowResize);
  }, [direction, size, minSize, maxSize, storageKey]);

  const containerStyle: React.CSSProperties = direction === 'horizontal'
    ? { width: size, minWidth: minSize, maxWidth: maxSize }
    : { height: size, minHeight: minSize, maxHeight: maxSize };

  return (
    <div
      ref={containerRef}
      className={`resizable-container resizable-container--${direction}`}
      style={containerStyle}
    >
      <div className="resizable-container__content">
        {children}
      </div>
      <div
        className={`resizable-container__handle resizable-container__handle--${direction}`}
        onMouseDown={handleMouseDown}
        title={direction === 'horizontal' ? 'Drag to resize width' : 'Drag to resize height'}
      >
        <div className="resizable-container__handle-indicator" />
      </div>
    </div>
  );
};

export default ResizableContainer;
