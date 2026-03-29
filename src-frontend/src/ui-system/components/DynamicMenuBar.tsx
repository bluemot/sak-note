import React, { useState, useRef, useEffect } from 'react';
import { UIComponentDefinition, uiRegistry, UISlot } from '../ModuleUIRegistry';
import { useCondition } from '../hooks/useCondition';
import { actionRegistry } from '../actions/actionRegistry';
import './DynamicMenuBar.css';

// Icon mapping for menu items
const iconMap: Record<string, string> = {
  'file': '📄',
  'edit': '✏️',
  'view': '👁️',
  'settings': '⚙️',
  'plugins': '🔌',
  'bookmarks': '📑',
  'marks': '🎨',
  'recent': '🕐',
  'open': '📂',
  'save': '💾',
  'undo': '↩️',
  'redo': '↪️',
  'cut': '✂️',
  'copy': '📋',
  'paste': '📌',
  'find': '🔍',
  'replace': '🔄',
  'new': '📄',
  'close': '❌',
  'exit': '🚪',
  'default': '•',
};

interface MenuItemProps {
  item: UIComponentDefinition;
  onClose: () => void;
}

const MenuItem: React.FC<MenuItemProps> = ({ item, onClose }) => {
  const isVisible = useCondition(item.when || 'always');
  const hasSubmenu = item.type === 'menu_submenu';
  const [isSubmenuOpen, setIsSubmenuOpen] = useState(false);
  const submenuRef = useRef<HTMLDivElement>(null);

  if (!isVisible) return null;

  const handleClick = async () => {
    if (item.action && !hasSubmenu) {
      try {
        await actionRegistry.execute(item.action);
      } catch (error) {
        // Error handled in actionRegistry
      }
      onClose();
    }
  };

  const handleMouseEnter = () => {
    if (hasSubmenu) {
      setIsSubmenuOpen(true);
    }
  };

  const handleMouseLeave = () => {
    if (hasSubmenu) {
      setIsSubmenuOpen(false);
    }
  };

  const icon = iconMap[item.icon || ''] || item.icon || iconMap['default'];

  // Get submenu items from registry
  const submenuItems = hasSubmenu
    ? uiRegistry.getComponentsForSlot(item.slot as UISlot).filter(
        c => c.group === item.id && c.visible !== false
      )
    : [];

  return (
    <div
      className={`menu-item-wrapper ${hasSubmenu ? 'has-submenu' : ''}`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <button
        className={`menu-item ${item.className || ''} ${hasSubmenu ? 'submenu-trigger' : ''}`}
        onClick={handleClick}
        disabled={!item.action && !hasSubmenu}
        data-action={item.action}
        data-module={item.module}
      >
        <span className="menu-item-icon">{icon}</span>
        <span className="menu-item-title">{item.title}</span>
        {item.shortcut && (
          <span className="menu-item-shortcut">{item.shortcut}</span>
        )}
        {hasSubmenu && <span className="submenu-arrow">▶</span>}
      </button>

      {hasSubmenu && isSubmenuOpen && submenuItems.length > 0 && (
        <div className="submenu" ref={submenuRef}>
          {submenuItems.map((subItem) => (
            <MenuItem
              key={`${subItem.module}:${subItem.id}`}
              item={subItem}
              onClose={onClose}
            />
          ))}
        </div>
      )}
    </div>
  );
};

interface MenuGroupProps {
  slot: UISlot;
  title: string;
  icon?: string;
  isOpen: boolean;
  onToggle: () => void;
  onClose: () => void;
}

const MenuGroup: React.FC<MenuGroupProps> = ({
  slot,
  title,
  icon,
  isOpen,
  onToggle,
  onClose,
}) => {
  const components = uiRegistry.getComponentsForSlot(slot);
  const menuRef = useRef<HTMLDivElement>(null);

  // Close when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen, onClose]);

  if (components.length === 0) return null;

  const menuIcon = iconMap[icon || ''] || icon || '';

  return (
    <div className="menu-group" ref={menuRef}>
      <button
        className={`menu-group-trigger ${isOpen ? 'open' : ''}`}
        onClick={onToggle}
        onMouseEnter={() => {
          if (document.querySelector('.menu-dropdown.open')) {
            onToggle();
          }
        }}
      >
        {menuIcon && <span className="menu-group-icon">{menuIcon}</span>}
        <span className="menu-group-title">{title}</span>
        <span className="menu-group-arrow">{isOpen ? '▲' : '▼'}</span>
      </button>

      {isOpen && (
        <div className="menu-dropdown open">
          {components.map((component) => (
            <MenuItem
              key={`${component.module}:${component.id}`}
              item={component}
              onClose={onClose}
            />
          ))}
        </div>
      )}
    </div>
  );
};

export const DynamicMenuBar: React.FC = () => {
  const [openMenu, setOpenMenu] = useState<string | null>(null);

  const handleToggle = (menuId: string) => {
    setOpenMenu(openMenu === menuId ? null : menuId);
  };

  const handleClose = () => {
    setOpenMenu(null);
  };

  // Keyboard shortcut support
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Alt key opens menu bar
      if (e.key === 'Alt' && !openMenu) {
        e.preventDefault();
        setOpenMenu('file');
      }
      // Escape closes menu
      if (e.key === 'Escape') {
        handleClose();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [openMenu]);

  return (
    <div className="dynamic-menubar">
      <div className="menubar-brand">
        <span className="brand-icon">🌸</span>
        <span className="brand-text">SAK Editor</span>
      </div>

      <div className="menubar-groups">
        <MenuGroup
          slot="menu.file"
          title="File"
          icon="file"
          isOpen={openMenu === 'file'}
          onToggle={() => handleToggle('file')}
          onClose={handleClose}
        />
        <MenuGroup
          slot="menu.editor"
          title="Editor"
          icon="editor"
          isOpen={openMenu === 'editor'}
          onToggle={() => handleToggle('editor')}
          onClose={handleClose}
        />
        <MenuGroup
          slot="menu.edit"
          title="Edit"
          icon="edit"
          isOpen={openMenu === 'edit'}
          onToggle={() => handleToggle('edit')}
          onClose={handleClose}
        />
        <MenuGroup
          slot="menu.view"
          title="View"
          icon="view"
          isOpen={openMenu === 'view'}
          onToggle={() => handleToggle('view')}
          onClose={handleClose}
        />
        <MenuGroup
          slot="menu.tools"
          title="Tools"
          icon="settings"
          isOpen={openMenu === 'tools'}
          onToggle={() => handleToggle('tools')}
          onClose={handleClose}
        />
        <MenuGroup
          slot="menu.plugins"
          title="Plugins"
          icon="plugins"
          isOpen={openMenu === 'plugins'}
          onToggle={() => handleToggle('plugins')}
          onClose={handleClose}
        />
      </div>

      <div className="menubar-spacer" />

      <div className="menubar-actions">
        <button
          className="menubar-action-btn"
          title="Minimize"
          onClick={() => {}}
        >
          🗕
        </button>
        <button
          className="menubar-action-btn"
          title="Maximize"
          onClick={() => {}}
        >
          🗖
        </button>
        <button
          className="menubar-action-btn close"
          title="Close"
          onClick={() => {}}
        >
          ✕
        </button>
      </div>
    </div>
  );
};

export default DynamicMenuBar;
