/**
 * Settings-related type definitions
 */

import type { VisualEffectsSettings } from './visualEffects';
import { defaultVisualEffectsSettings } from './visualEffects';

export interface AppSettings {
  general: GeneralSettings;
  terminal: TerminalSettings;
  appearance: AppearanceSettings;
  keyboard: KeyboardSettings;
  notifications: NotificationSettings;
  visualEffects: VisualEffectsSettings;
}

export interface GeneralSettings {
  startMinimized: boolean;
  minimizeToTray: boolean;
  checkUpdates: boolean;
  autoConnect: boolean;
  language: string;
}

export interface TerminalSettings {
  fontFamily: string;
  fontSize: number;
  lineHeight: number;
  cursorStyle: 'block' | 'underline' | 'bar';
  cursorBlink: boolean;
  scrollback: number;
  copyOnSelect: boolean;
  rightClickPaste: boolean;
  bellSound: boolean;
  theme: string;
}

export interface AppearanceSettings {
  theme: 'light' | 'dark' | 'system';
  accentColor: string;
  terminalTheme: string;
  sidebarPosition: 'left' | 'right';
  compactMode: boolean;
  compactSidebar: boolean;
  showStatusBar: boolean;
}

export interface KeyboardSettings {
  shortcuts: Record<string, string>;
  enableGlobalShortcuts: boolean;
}

export interface NotificationSettings {
  enabled: boolean;
  sound: boolean;
  connectionEvents: boolean;
  transferComplete: boolean;
  errors: boolean;
}

export const defaultSettings: AppSettings = {
  general: {
    startMinimized: false,
    minimizeToTray: true,
    checkUpdates: true,
    autoConnect: false,
    language: 'en',
  },
  terminal: {
    fontFamily: 'JetBrains Mono, Menlo, Monaco, monospace',
    fontSize: 14,
    lineHeight: 1.2,
    cursorStyle: 'block',
    cursorBlink: true,
    scrollback: 10000,
    copyOnSelect: false,
    rightClickPaste: true,
    bellSound: false,
    theme: 'dark',
  },
  appearance: {
    theme: 'system',
    accentColor: '#3b82f6',
    terminalTheme: 'dracula',
    sidebarPosition: 'left',
    compactMode: false,
    compactSidebar: false,
    showStatusBar: true,
  },
  keyboard: {
    shortcuts: {
      newConnection: 'Ctrl+N',
      newTab: 'Ctrl+T',
      closeTab: 'Ctrl+W',
      nextTab: 'Ctrl+Tab',
      prevTab: 'Ctrl+Shift+Tab',
      commandPalette: 'Ctrl+K',
      settings: 'Ctrl+,',
      toggleSidebar: 'Ctrl+B',
    },
    enableGlobalShortcuts: false,
  },
  notifications: {
    enabled: true,
    sound: false,
    connectionEvents: true,
    transferComplete: true,
    errors: true,
  },
  visualEffects: defaultVisualEffectsSettings,
};
