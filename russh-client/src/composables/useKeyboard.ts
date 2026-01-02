/**
 * Keyboard composable - handles global keyboard shortcuts
 */

import { onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useSettingsStore } from '@/stores/settings';
import { useTerminalStore } from '@/stores/terminals';

export function useKeyboard() {
  const router = useRouter();
  const settingsStore = useSettingsStore();
  const terminalStore = useTerminalStore();

  const actions: Record<string, () => void> = {
    newConnection: () => router.push('/connections/new'),
    newTab: () => {
      // Handled by terminal view
      document.dispatchEvent(new CustomEvent('new-terminal-tab'));
    },
    closeTab: () => {
      if (terminalStore.activeTabId) {
        terminalStore.closeTab(terminalStore.activeTabId);
      }
    },
    nextTab: () => terminalStore.nextTab(),
    prevTab: () => terminalStore.previousTab(),
    commandPalette: () => {
      document.dispatchEvent(new CustomEvent('open-command-palette'));
    },
    settings: () => router.push('/settings'),
    toggleSidebar: () => {
      document.dispatchEvent(new CustomEvent('toggle-sidebar'));
    },
  };

  function handleKeydown(event: KeyboardEvent) {
    // Don't intercept when typing in inputs (except for specific shortcuts)
    const target = event.target as HTMLElement;
    const isInput = target instanceof HTMLInputElement || 
                    target instanceof HTMLTextAreaElement ||
                    target.isContentEditable;
    
    // Allow command palette shortcut even in inputs
    const isCommandPalette = (event.ctrlKey || event.metaKey) && event.key === 'k';
    
    if (isInput && !isCommandPalette) {
      return;
    }

    const shortcuts = settingsStore.settings.keyboard.shortcuts;
    const pressed = getShortcutString(event);

    for (const [action, shortcut] of Object.entries(shortcuts)) {
      if (normalizeShortcut(shortcut) === pressed) {
        event.preventDefault();
        event.stopPropagation();
        actions[action]?.();
        return;
      }
    }
  }

  function getShortcutString(event: KeyboardEvent): string {
    const parts: string[] = [];
    if (event.ctrlKey || event.metaKey) parts.push('ctrl');
    if (event.shiftKey) parts.push('shift');
    if (event.altKey) parts.push('alt');
    
    // Normalize key
    let key = event.key.toLowerCase();
    if (key === ' ') key = 'space';
    if (key === 'escape') key = 'esc';
    
    parts.push(key);
    return parts.sort().join('+');
  }

  function normalizeShortcut(shortcut: string): string {
    return shortcut
      .toLowerCase()
      .replace(/cmd/g, 'ctrl')
      .replace(/command/g, 'ctrl')
      .replace(/\s+/g, '')
      .split('+')
      .sort()
      .join('+');
  }

  function formatShortcut(shortcut: string): string {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    
    return shortcut
      .split('+')
      .map(key => {
        const k = key.trim().toLowerCase();
        if (k === 'ctrl') return isMac ? '⌘' : 'Ctrl';
        if (k === 'shift') return isMac ? '⇧' : 'Shift';
        if (k === 'alt') return isMac ? '⌥' : 'Alt';
        if (k === 'tab') return '⇥';
        if (k === 'esc') return 'Esc';
        return k.charAt(0).toUpperCase() + k.slice(1);
      })
      .join(isMac ? '' : '+');
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown);
  });

  return {
    getShortcutString,
    normalizeShortcut,
    formatShortcut,
  };
}
