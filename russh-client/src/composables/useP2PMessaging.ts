/**
 * P2P Messaging composable - handles real-time message streaming
 */

import { ref, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useP2PTerminalStore } from '@/stores/p2pTerminal';
import { useNotificationStore } from '@/stores/notifications';
import type { 
  Block, 
  TextBlock, 
  CodeBlock, 
  FileBlock, 
  WidgetBlock,
  SystemBlock,
  P2PMessage,
  WidgetConfig,
  WidgetResponse 
} from '@/types/blocks';
import { parseBackendError } from '@/types/errors';

function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

export function useP2PMessaging() {
  const terminalStore = useP2PTerminalStore();
  const notificationStore = useNotificationStore();

  const isConnected = ref(false);
  const localNodeId = ref<string>('');
  
  let unlistenMessage: UnlistenFn | null = null;
  let unlistenTyping: UnlistenFn | null = null;
  let typingTimeout: ReturnType<typeof setTimeout> | null = null;

  async function initialize(nodeId: string) {
    localNodeId.value = nodeId;
    
    // Listen for incoming messages
    unlistenMessage = await listen<{ peerId: string; message: P2PMessage }>(
      'p2p-message',
      (event) => {
        handleIncomingMessage(event.payload.peerId, event.payload.message);
      }
    );

    // Listen for typing indicators
    unlistenTyping = await listen<{ peerId: string; isTyping: boolean }>(
      'p2p-typing',
      (event) => {
        terminalStore.setTypingIndicator(event.payload.peerId, event.payload.isTyping);
      }
    );

    isConnected.value = true;
  }

  function handleIncomingMessage(peerId: string, message: P2PMessage) {
    switch (message.type) {
      case 'block':
        const block = message.payload as Block;
        block.isLocal = false;
        terminalStore.addBlock(peerId, block);
        break;
      case 'typing':
        // Handled by separate listener
        break;
      case 'ack':
        terminalStore.removePendingMessage(message.id);
        break;
      case 'widget-response':
        const { blockId, response } = message.payload as { blockId: string; response: WidgetResponse };
        terminalStore.addWidgetResponse(peerId, blockId, response);
        break;
    }
  }

  async function sendTextBlock(peerId: string, content: string, format: 'plain' | 'markdown' = 'plain') {
    const block: TextBlock = {
      id: generateId(),
      type: 'text',
      senderId: localNodeId.value,
      timestamp: Date.now(),
      isLocal: true,
      content,
      format,
    };

    terminalStore.addBlock(peerId, block);
    await sendBlock(peerId, block);
  }

  async function sendCodeBlock(peerId: string, content: string, language: string, filename?: string) {
    const block: CodeBlock = {
      id: generateId(),
      type: 'code',
      senderId: localNodeId.value,
      timestamp: Date.now(),
      isLocal: true,
      content,
      language,
      filename,
      showLineNumbers: true,
    };

    terminalStore.addBlock(peerId, block);
    await sendBlock(peerId, block);
  }

  async function sendFileBlock(peerId: string, file: File) {
    const block: FileBlock = {
      id: generateId(),
      type: 'file',
      senderId: localNodeId.value,
      timestamp: Date.now(),
      isLocal: true,
      filename: file.name,
      size: file.size,
      mimeType: file.type,
      transferStatus: 'pending',
      transferProgress: 0,
    };

    // Generate preview for images
    let previewUrl: string | undefined;
    if (file.type.startsWith('image/')) {
      previewUrl = URL.createObjectURL(file);
      block.previewUrl = previewUrl;
    }

    terminalStore.addBlock(peerId, block);

    try {
      // Read file data
      const arrayBuffer = await file.arrayBuffer();
      block.data = arrayBuffer;
      block.transferStatus = 'transferring';
      terminalStore.updateBlock(peerId, block.id, { transferStatus: 'transferring' });

      await sendBlock(peerId, block);

      terminalStore.updateBlock(peerId, block.id, { 
        transferStatus: 'completed',
        transferProgress: 100 
      });
    } catch (e) {
      terminalStore.updateBlock(peerId, block.id, { transferStatus: 'failed' });
      // Clean up preview URL on failure
      if (previewUrl) {
        URL.revokeObjectURL(previewUrl);
      }
      throw e;
    }
  }

  async function sendWidgetBlock(peerId: string, widgetType: string, config: WidgetConfig) {
    const block: WidgetBlock = {
      id: generateId(),
      type: 'widget',
      senderId: localNodeId.value,
      timestamp: Date.now(),
      isLocal: true,
      widgetType: widgetType as WidgetBlock['widgetType'],
      config,
      responses: [],
    };

    terminalStore.addBlock(peerId, block);
    await sendBlock(peerId, block);
  }

  async function sendSystemBlock(peerId: string, message: string, level: SystemBlock['level'] = 'info') {
    const block: SystemBlock = {
      id: generateId(),
      type: 'system',
      senderId: localNodeId.value,
      timestamp: Date.now(),
      isLocal: true,
      message,
      level,
    };

    terminalStore.addBlock(peerId, block);
  }

  async function sendBlock(peerId: string, block: Block) {
    const message: P2PMessage = {
      id: generateId(),
      type: 'block',
      payload: block,
      timestamp: Date.now(),
    };

    terminalStore.addPendingMessage(message);

    try {
      await invoke('p2p_send_message', { 
        peerId, 
        message: JSON.stringify(message) 
      });
    } catch (e) {
      const appError = parseBackendError(e);
      notificationStore.error('Send Failed', appError.message);
      terminalStore.removePendingMessage(message.id);
      throw e;
    }
  }

  async function sendTypingIndicator(peerId: string, isTyping: boolean) {
    // Debounce typing indicator
    if (typingTimeout) {
      clearTimeout(typingTimeout);
    }

    if (isTyping) {
      typingTimeout = setTimeout(() => {
        sendTypingIndicator(peerId, false);
      }, 3000);
    }

    try {
      await invoke('p2p_send_typing', { peerId, isTyping });
    } catch (e) {
      // Silently fail for typing indicators
      console.warn('Failed to send typing indicator:', e);
    }
  }

  async function sendWidgetResponse(peerId: string, blockId: string, value: unknown) {
    const response: WidgetResponse = {
      responderId: localNodeId.value,
      timestamp: Date.now(),
      value,
    };

    const message: P2PMessage = {
      id: generateId(),
      type: 'widget-response',
      payload: { blockId, response },
      timestamp: Date.now(),
    };

    try {
      await invoke('p2p_send_message', { 
        peerId, 
        message: JSON.stringify(message) 
      });
      
      // Also add to local store
      terminalStore.addWidgetResponse(peerId, blockId, response);
    } catch (e) {
      const appError = parseBackendError(e);
      notificationStore.error('Response Failed', appError.message);
      throw e;
    }
  }

  function dispose() {
    unlistenMessage?.();
    unlistenTyping?.();
    if (typingTimeout) {
      clearTimeout(typingTimeout);
    }
    isConnected.value = false;
  }

  onUnmounted(dispose);

  return {
    // State
    isConnected,
    localNodeId,
    // Actions
    initialize,
    sendTextBlock,
    sendCodeBlock,
    sendFileBlock,
    sendWidgetBlock,
    sendSystemBlock,
    sendTypingIndicator,
    sendWidgetResponse,
    dispose,
  };
}
