/**
 * SSH composable - wraps Tauri invoke calls for SSH operations
 */

import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useNotificationStore } from '@/stores/notifications';
import type { CommandResult, ConnectionRequest } from '@/types/ssh';
import { parseBackendError } from '@/types/errors';

export function useSSH(sessionId?: string) {
  const notificationStore = useNotificationStore();
  
  const isExecuting = ref(false);
  const isConnecting = ref(false);
  const lastResult = ref<CommandResult | null>(null);
  const error = ref<string | null>(null);
  const currentSessionId = ref(sessionId || '');

  async function connect(request: ConnectionRequest): Promise<string> {
    isConnecting.value = true;
    error.value = null;
    
    try {
      const response = await invoke<{ sessionId: string }>('ssh_connect', { request });
      currentSessionId.value = response.sessionId;
      notificationStore.success('Connected', `Connected to ${request.host}`);
      return response.sessionId;
    } catch (e) {
      const appError = parseBackendError(e);
      error.value = appError.message;
      notificationStore.error('Connection Failed', appError.message);
      throw e;
    } finally {
      isConnecting.value = false;
    }
  }

  async function execute(command: string, timeout?: number): Promise<CommandResult> {
    isExecuting.value = true;
    error.value = null;
    
    try {
      const result = await invoke<CommandResult>('ssh_execute', {
        request: {
          sessionId: currentSessionId.value,
          command,
          timeoutSecs: timeout,
        }
      });
      lastResult.value = result;
      return result;
    } catch (e) {
      const appError = parseBackendError(e);
      error.value = appError.message;
      notificationStore.error('Command Failed', appError.message);
      throw e;
    } finally {
      isExecuting.value = false;
    }
  }

  async function disconnect() {
    try {
      await invoke('ssh_disconnect', { sessionId: currentSessionId.value });
      notificationStore.info('Disconnected', 'SSH session closed');
    } catch (e) {
      const appError = parseBackendError(e);
      notificationStore.error('Disconnect Failed', appError.message);
      throw e;
    }
  }

  return {
    isExecuting,
    isConnecting,
    lastResult,
    error,
    sessionId: currentSessionId,
    connect,
    execute,
    disconnect,
  };
}
