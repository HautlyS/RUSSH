/**
 * Terminal composable - manages xterm.js terminal instances
 */

import { ref, shallowRef, onUnmounted, watch } from 'vue';
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { WebglAddon } from 'xterm-addon-webgl';
import { SearchAddon } from 'xterm-addon-search';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useSettingsStore } from '@/stores/settings';
import { getTerminalTheme } from '@/utils/terminalThemes';

export interface TerminalOptions {
  fontSize?: number;
  fontFamily?: string;
  theme?: string;
  cursorStyle?: 'block' | 'underline' | 'bar';
  cursorBlink?: boolean;
  scrollback?: number;
}

export function useTerminal() {
  const settingsStore = useSettingsStore();
  
  const terminal = shallowRef<Terminal | null>(null);
  const fitAddon = shallowRef<FitAddon | null>(null);
  const searchAddon = shallowRef<SearchAddon | null>(null);
  const isReady = ref(false);
  const sessionId = ref<string>('');
  
  let unlistenOutput: UnlistenFn | null = null;

  async function initTerminal(container: HTMLElement, options?: TerminalOptions) {
    // Dispose existing terminal if any
    destroyTerminal();
    
    const settings = settingsStore.settings.terminal;
    
    terminal.value = new Terminal({
      fontFamily: options?.fontFamily || settings.fontFamily,
      fontSize: options?.fontSize || settings.fontSize,
      lineHeight: settings.lineHeight,
      cursorStyle: options?.cursorStyle || settings.cursorStyle,
      cursorBlink: options?.cursorBlink ?? settings.cursorBlink,
      scrollback: options?.scrollback || settings.scrollback,
      theme: getTerminalTheme(options?.theme || settings.theme || 'dark'),
      allowProposedApi: true,
    });

    // Load addons
    fitAddon.value = new FitAddon();
    terminal.value.loadAddon(fitAddon.value);

    searchAddon.value = new SearchAddon();
    terminal.value.loadAddon(searchAddon.value);

    // Try WebGL, fall back to canvas
    try {
      const webglAddon = new WebglAddon();
      webglAddon.onContextLoss(() => webglAddon.dispose());
      terminal.value.loadAddon(webglAddon);
    } catch (e) {
      console.warn('WebGL not available, using canvas renderer');
    }

    terminal.value.open(container);
    fitAddon.value.fit();
    isReady.value = true;
  }

  async function attachToSession(sid: string) {
    if (!terminal.value) return;
    
    sessionId.value = sid;

    // Handle input
    terminal.value.onData(async (data) => {
      try {
        await invoke('terminal_input', { sessionId: sid, data });
      } catch (e) {
        console.error('Failed to send terminal input:', e);
      }
    });

    // Handle resize
    terminal.value.onResize(async ({ cols, rows }) => {
      try {
        await invoke('terminal_resize', { sessionId: sid, cols, rows });
      } catch (e) {
        console.error('Failed to resize terminal:', e);
      }
    });

    // Listen for output from backend
    unlistenOutput = await listen<string>(`terminal-output-${sid}`, (event) => {
      terminal.value?.write(event.payload);
    });

    // Start PTY session
    try {
      await invoke('terminal_start', { sessionId: sid });
    } catch (e) {
      console.error('Failed to start terminal:', e);
    }
  }

  function write(data: string) {
    terminal.value?.write(data);
  }

  function resize() {
    fitAddon.value?.fit();
  }

  function focus() {
    terminal.value?.focus();
  }

  function clear() {
    terminal.value?.clear();
  }

  function copySelection(): string {
    const selection = terminal.value?.getSelection() || '';
    if (selection) {
      navigator.clipboard.writeText(selection);
    }
    return selection;
  }

  function paste(text: string) {
    terminal.value?.paste(text);
  }

  function searchNext(query: string) {
    searchAddon.value?.findNext(query);
  }

  function searchPrevious(query: string) {
    searchAddon.value?.findPrevious(query);
  }

  function destroyTerminal() {
    unlistenOutput?.();
    unlistenOutput = null;
    terminal.value?.dispose();
    terminal.value = null;
    fitAddon.value = null;
    searchAddon.value = null;
    isReady.value = false;
  }

  // Watch for settings changes
  watch(
    () => settingsStore.settings.terminal,
    (newSettings) => {
      if (!terminal.value) return;
      terminal.value.options.fontFamily = newSettings.fontFamily;
      terminal.value.options.fontSize = newSettings.fontSize;
      terminal.value.options.lineHeight = newSettings.lineHeight;
      terminal.value.options.cursorStyle = newSettings.cursorStyle;
      terminal.value.options.cursorBlink = newSettings.cursorBlink;
      fitAddon.value?.fit();
    },
    { deep: true }
  );

  onUnmounted(destroyTerminal);

  return {
    terminal,
    isReady,
    initTerminal,
    attachToSession,
    write,
    resize,
    focus,
    clear,
    copySelection,
    paste,
    searchNext,
    searchPrevious,
    destroyTerminal,
  };
}
