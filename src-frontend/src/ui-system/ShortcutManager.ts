/**
 * ShortcutManager
 *
 * Global keyboard shortcut management with conflict detection
 * and shortcut hint display.
 */

import { uiRegistry } from './ModuleUIRegistry';
import { notify } from './components/NotificationContainer';

// Shortcut entry
interface ShortcutEntry {
  shortcut: string;
  action: string;
  module?: string;
  description?: string;
  enabled: boolean;
  preventDefault: boolean;
}

// Normalized shortcut format
interface NormalizedShortcut {
  key: string;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  meta: boolean;
}

export class ShortcutManager {
  private shortcuts: Map<string, ShortcutEntry> = new Map();
  private conflicts: Map<string, string[]> = new Map();
  private isInitialized: boolean = false;
  private disabledElements: Set<HTMLElement> = new Set();

  /**
   * Initialize the shortcut manager and attach global listeners
   */
  init(): void {
    if (this.isInitialized) return;

    // Register built-in shortcuts from UI registry
    this.registerFromRegistry();

    // Listen for registry changes
    window.addEventListener('ui-registry-change', () => {
      this.registerFromRegistry();
    });

    // Attach global keydown listener
    document.addEventListener('keydown', this.handleKeyDown, true);

    this.isInitialized = true;
    console.log('[ShortcutManager] Initialized');
  }

  /**
   * Cleanup and remove listeners
   */
  destroy(): void {
    document.removeEventListener('keydown', this.handleKeyDown, true);
    window.removeEventListener('ui-registry-change', this.registerFromRegistry);
    this.isInitialized = false;
    console.log('[ShortcutManager] Destroyed');
  }

  /**
   * Register a shortcut
   * @param shortcut Shortcut string (e.g., "Ctrl+Shift+A")
   * @param action Action ID to execute
   * @param options Optional configuration
   */
  register(
    shortcut: string,
    action: string,
    options?: {
      module?: string;
      description?: string;
      enabled?: boolean;
      preventDefault?: boolean;
    }
  ): void {
    const normalizedShortcut = this.normalizeShortcutString(shortcut);
    if (!normalizedShortcut) {
      console.warn(`[ShortcutManager] Invalid shortcut: ${shortcut}`);
      return;
    }

    const shortcutKey = this.shortcutToKey(normalizedShortcut);
    const entry: ShortcutEntry = {
      shortcut: shortcutKey,
      action,
      module: options?.module,
      description: options?.description,
      enabled: options?.enabled ?? true,
      preventDefault: options?.preventDefault ?? true,
    };

    // Check for conflicts
    if (this.shortcuts.has(shortcutKey)) {
      const existing = this.shortcuts.get(shortcutKey)!;
      if (!this.conflicts.has(shortcutKey)) {
        this.conflicts.set(shortcutKey, [existing.action]);
      }
      this.conflicts.get(shortcutKey)!.push(action);
      console.warn(
        `[ShortcutManager] Conflict detected: ${shortcut} already bound to ${existing.action}, now also ${action}`
      );
    }

    this.shortcuts.set(shortcutKey, entry);
    console.log(`[ShortcutManager] Registered: ${shortcut} -> ${action}`);
  }

  /**
   * Unregister a shortcut
   * @param shortcut Shortcut string to unregister
   */
  unregister(shortcut: string): void {
    const normalized = this.normalizeShortcutString(shortcut);
    if (!normalized) return;

    const shortcutKey = this.shortcutToKey(normalized);
    this.shortcuts.delete(shortcutKey);

    // Clean up conflicts
    if (this.conflicts.has(shortcutKey)) {
      this.conflicts.delete(shortcutKey);
    }

    console.log(`[ShortcutManager] Unregistered: ${shortcut}`);
  }

  /**
   * Get shortcut for an action
   * @param action Action ID
   * @returns Shortcut string or undefined
   */
  getShortcutForAction(action: string): string | undefined {
    for (const [key, entry] of this.shortcuts) {
      if (entry.action === action && entry.enabled) {
        return this.keyToDisplayString(key);
      }
    }
    return undefined;
  }

  /**
   * Get action for a shortcut
   * @param shortcut Shortcut string
   * @returns Action ID or undefined
   */
  getActionForShortcut(shortcut: string): string | undefined {
    const normalized = this.normalizeShortcutString(shortcut);
    if (!normalized) return undefined;

    const shortcutKey = this.shortcutToKey(normalized);
    const entry = this.shortcuts.get(shortcutKey);
    return entry?.enabled ? entry.action : undefined;
  }

  /**
   * Enable/disable a shortcut
   * @param shortcut Shortcut string
   * @param enabled Whether to enable or disable
   */
  setEnabled(shortcut: string, enabled: boolean): void {
    const normalized = this.normalizeShortcutString(shortcut);
    if (!normalized) return;

    const shortcutKey = this.shortcutToKey(normalized);
    const entry = this.shortcuts.get(shortcutKey);
    if (entry) {
      entry.enabled = enabled;
    }
  }

  /**
   * Check if a shortcut is registered
   * @param shortcut Shortcut string
   */
  isRegistered(shortcut: string): boolean {
    const normalized = this.normalizeShortcutString(shortcut);
    if (!normalized) return false;

    const shortcutKey = this.shortcutToKey(normalized);
    return this.shortcuts.has(shortcutKey);
  }

  /**
   * Get all registered shortcuts
   */
  getAllShortcuts(): ShortcutEntry[] {
    return Array.from(this.shortcuts.values());
  }

  /**
   * Get all conflicts
   */
  getConflicts(): Map<string, string[]> {
    return new Map(this.conflicts);
  }

  /**
   * Show shortcut hint
   * @param shortcut Shortcut to show hint for
   */
  showHint(shortcut: string): void {
    const action = this.getActionForShortcut(shortcut);
    if (action) {
      notify.info(`Shortcut: ${shortcut} → ${action}`);
    }
  }

  /**
   * Disable shortcuts for a specific element (e.g., input field)
   * @param element Element to disable shortcuts for
   */
  disableForElement(element: HTMLElement): void {
    this.disabledElements.add(element);
  }

  /**
   * Re-enable shortcuts for an element
   * @param element Element to re-enable
   */
  enableForElement(element: HTMLElement): void {
    this.disabledElements.delete(element);
  }

  /**
   * Handle keydown event
   */
  private handleKeyDown = (event: KeyboardEvent): void => {
    // Skip if target is in a disabled element
    const target = event.target as HTMLElement;
    if (this.shouldIgnoreEvent(target)) {
      return;
    }

    const shortcut = this.eventToShortcut(event);
    const shortcutKey = this.shortcutToKey(shortcut);
    const entry = this.shortcuts.get(shortcutKey);

    if (entry && entry.enabled) {
      // Prevent default if configured
      if (entry.preventDefault) {
        event.preventDefault();
        event.stopPropagation();
      }

      // Execute action
      this.executeAction(entry.action);
    }
  };

  /**
   * Execute an action
   */
  private async executeAction(action: string): Promise<void> {
    console.log(`[ShortcutManager] Executing action: ${action}`);

    try {
      // Use uiRegistry to execute action
      await uiRegistry.executeAction(action);
    } catch (err) {
      console.error(`[ShortcutManager] Action execution failed: ${action}`, err);
    }
  }

  /**
   * Check if we should ignore keyboard event
   */
  private shouldIgnoreEvent(target: HTMLElement): boolean {
    // Ignore if target is in a disabled element
    if (this.disabledElements.has(target)) {
      return true;
    }

    // Ignore if target is an input, textarea, or contenteditable
    const tagName = target.tagName.toLowerCase();
    if (tagName === 'input' || tagName === 'textarea' || tagName === 'select') {
      return true;
    }

    if (target.isContentEditable) {
      return true;
    }

    return false;
  }

  /**
   * Convert keyboard event to normalized shortcut
   */
  private eventToShortcut(event: KeyboardEvent): NormalizedShortcut {
    // Get the key, handling special cases
    let key = event.key;

    // Normalize special keys
    if (key === ' ') {
      key = 'Space';
    } else if (key.length === 1) {
      // Single character - uppercase for consistency
      key = key.toUpperCase();
    }

    return {
      key,
      ctrl: event.ctrlKey,
      shift: event.shiftKey,
      alt: event.altKey,
      meta: event.metaKey,
    };
  }

  /**
   * Normalize shortcut string
   */
  private normalizeShortcutString(shortcut: string): NormalizedShortcut | null {
    const parts = shortcut.split('+').map((p) => p.trim().toLowerCase());

    const result: NormalizedShortcut = {
      key: '',
      ctrl: false,
      shift: false,
      alt: false,
      meta: false,
    };

    for (const part of parts) {
      switch (part) {
        case 'ctrl':
        case 'control':
          result.ctrl = true;
          break;
        case 'shift':
          result.shift = true;
          break;
        case 'alt':
          result.alt = true;
          break;
        case 'meta':
        case 'cmd':
        case 'command':
        case 'win':
        case 'windows':
          result.meta = true;
          break;
        default:
          result.key = part.toUpperCase();
          break;
      }
    }

    if (!result.key) {
      return null;
    }

    return result;
  }

  /**
   * Convert normalized shortcut to storage key
   */
  private shortcutToKey(shortcut: NormalizedShortcut): string {
    const parts: string[] = [];
    if (shortcut.ctrl) parts.push('Ctrl');
    if (shortcut.shift) parts.push('Shift');
    if (shortcut.alt) parts.push('Alt');
    if (shortcut.meta) parts.push('Meta');
    parts.push(shortcut.key);
    return parts.join('+');
  }

  /**
   * Convert storage key to display string
   */
  private keyToDisplayString(key: string): string {
    return key;
  }

  /**
   * Register shortcuts from UI registry
   */
  private registerFromRegistry = (): void => {
    const shortcuts = uiRegistry.getShortcuts();
    for (const shortcut of shortcuts) {
      const shortcutString = this.buildShortcutString(shortcut);
      this.register(shortcutString, shortcut.action, {
        module: 'registry',
        description: shortcut.action,
        preventDefault: true,
      });
    }
  };

  /**
   * Build shortcut string from registry definition
   */
  private buildShortcutString(shortcut: { key: string; modifiers?: string[] }): string {
    const parts: string[] = [];
    if (shortcut.modifiers) {
      parts.push(...shortcut.modifiers);
    }
    parts.push(shortcut.key);
    return parts.join('+');
  }
}

// Global instance
export const shortcutManager = new ShortcutManager();

export default ShortcutManager;
