/**
 * Background connection composable - keeps SSH connections alive in background
 */

import { ref, onMounted, onUnmounted, watch } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { usePlatform } from './usePlatform';
import { useConnectionStore } from '@/stores/connections';
import { useNotificationStore } from '@/stores/notifications';

export type AppState = 'active' | 'inactive' | 'background';

export function useBackgroundConnection() {
  const { isMobile, isTauri } = usePlatform();
  const connectionStore = useConnectionStore();
  const notificationStore = useNotificationStore();
  
  const appState = ref<AppState>('active');
  const keepAliveInterval = ref<number | null>(null);
  
  let unlistenForeground: UnlistenFn | null = null;
  let unlistenBackground: UnlistenFn | null = null;
  let visibilityHandler: (() => void) | null = null;

  // Send keep-alive to all active connections
  async function sendKeepAlive() {
    const activeConnections = Array.from(connectionStore.activeConnections.values())
      .filter(c => c.status === 'connected');
    
    for (const conn of activeConnections) {
      try {
        await invoke('ssh_keep_alive', { sessionId: conn.sessionId });
      } catch (e) {
        // Connection may have dropped
        console.warn(`Keep-alive failed for ${conn.sessionId}:`, e);
        
        // Notify user if in background
        if (appState.value === 'background') {
          notificationStore.warning('Connection Lost', 'Connection to server was lost');
        }
      }
    }
  }

  // Start keep-alive timer
  function startKeepAlive() {
    if (keepAliveInterval.value) return;
    
    // Send keep-alive every 30 seconds
    keepAliveInterval.value = window.setInterval(sendKeepAlive, 30000);
  }

  // Stop keep-alive timer
  function stopKeepAlive() {
    if (keepAliveInterval.value) {
      clearInterval(keepAliveInterval.value);
      keepAliveInterval.value = null;
    }
  }

  // Handle app going to foreground
  async function onForeground() {
    appState.value = 'active';
    
    // Verify all connections are still alive
    const activeConnections = Array.from(connectionStore.activeConnections.values())
      .filter(c => c.status === 'connected');
    
    for (const conn of activeConnections) {
      try {
        await invoke('ssh_check_connection', { sessionId: conn.sessionId });
      } catch {
        // Connection dropped while in background
        connectionStore.activeConnections.delete(conn.profileId);
        notificationStore.warning('Connection Lost', 'A connection was lost while the app was in the background');
      }
    }
  }

  // Handle app going to background
  function onBackground() {
    appState.value = 'background';
    
    // Start more frequent keep-alive in background
    stopKeepAlive();
    keepAliveInterval.value = window.setInterval(sendKeepAlive, 15000);
  }

  // Watch for active connections
  watch(
    () => connectionStore.connectedProfiles.length,
    (count) => {
      if (count > 0) {
        startKeepAlive();
      } else {
        stopKeepAlive();
      }
    },
    { immediate: true }
  );

  onMounted(async () => {
    if (!isTauri.value) return;

    // Listen for app lifecycle events
    unlistenForeground = await listen('app-foreground', onForeground);
    unlistenBackground = await listen('app-background', onBackground);

    // Also use visibility API as fallback
    visibilityHandler = () => {
      if (document.visibilityState === 'visible') {
        onForeground();
      } else {
        onBackground();
      }
    };
    document.addEventListener('visibilitychange', visibilityHandler);
  });

  onUnmounted(() => {
    stopKeepAlive();
    unlistenForeground?.();
    unlistenBackground?.();
    if (visibilityHandler) {
      document.removeEventListener('visibilitychange', visibilityHandler);
    }
  });

  return {
    appState,
    sendKeepAlive,
    startKeepAlive,
    stopKeepAlive,
  };
}
