/**
 * P2P Terminal store - manages conversations and blocks
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Block, P2PMessage, TypingIndicator, WidgetResponse } from '@/types/blocks';
import type { P2PPeer } from '@/types/p2p';

export interface Conversation {
  peerId: string;
  peerName?: string;
  blocks: Block[];
  unreadCount: number;
  lastActivity: number;
  isTyping: boolean;
}

export const useP2PTerminalStore = defineStore('p2pTerminal', () => {
  // State
  const conversations = ref<Map<string, Conversation>>(new Map());
  const activeConversationId = ref<string | null>(null);
  const typingPeers = ref<Map<string, TypingIndicator>>(new Map());
  const pendingMessages = ref<Map<string, P2PMessage>>(new Map());

  // Getters
  const activeConversation = computed(() => {
    if (!activeConversationId.value) return null;
    return conversations.value.get(activeConversationId.value) || null;
  });

  const conversationList = computed(() => {
    return Array.from(conversations.value.values())
      .sort((a, b) => b.lastActivity - a.lastActivity);
  });

  const totalUnread = computed(() => {
    return Array.from(conversations.value.values())
      .reduce((sum, conv) => sum + conv.unreadCount, 0);
  });

  // Actions
  function createConversation(peer: P2PPeer): Conversation {
    const existing = conversations.value.get(peer.id);
    if (existing) return existing;

    const conversation: Conversation = {
      peerId: peer.id,
      peerName: peer.name,
      blocks: [],
      unreadCount: 0,
      lastActivity: Date.now(),
      isTyping: false,
    };

    conversations.value.set(peer.id, conversation);
    return conversation;
  }

  function setActiveConversation(peerId: string | null) {
    activeConversationId.value = peerId;
    if (peerId) {
      const conv = conversations.value.get(peerId);
      if (conv) {
        conv.unreadCount = 0;
      }
    }
  }

  function addBlock(peerId: string, block: Block) {
    let conv = conversations.value.get(peerId);
    if (!conv) {
      conv = {
        peerId,
        blocks: [],
        unreadCount: 0,
        lastActivity: Date.now(),
        isTyping: false,
      };
      conversations.value.set(peerId, conv);
    }

    conv.blocks.push(block);
    conv.lastActivity = block.timestamp;

    // Increment unread if not active conversation
    if (activeConversationId.value !== peerId && !block.isLocal) {
      conv.unreadCount++;
    }
  }

  function updateBlock(peerId: string, blockId: string, updates: Partial<Block>) {
    const conv = conversations.value.get(peerId);
    if (!conv) return;

    const blockIndex = conv.blocks.findIndex(b => b.id === blockId);
    if (blockIndex !== -1) {
      conv.blocks[blockIndex] = { ...conv.blocks[blockIndex], ...updates } as Block;
    }
  }

  function addWidgetResponse(peerId: string, blockId: string, response: WidgetResponse) {
    const conv = conversations.value.get(peerId);
    if (!conv) return;

    const block = conv.blocks.find(b => b.id === blockId);
    if (block && block.type === 'widget') {
      if (!block.responses) block.responses = [];
      block.responses.push(response);
    }
  }

  function setTypingIndicator(peerId: string, isTyping: boolean) {
    const conv = conversations.value.get(peerId);
    if (conv) {
      conv.isTyping = isTyping;
    }
  }

  function clearConversation(peerId: string) {
    const conv = conversations.value.get(peerId);
    if (conv) {
      // Clean up any object URLs from file blocks
      conv.blocks.forEach(block => {
        if (block.type === 'file' && block.previewUrl) {
          URL.revokeObjectURL(block.previewUrl);
        }
      });
      conv.blocks = [];
      conv.unreadCount = 0;
    }
  }

  function removeConversation(peerId: string) {
    const conv = conversations.value.get(peerId);
    if (conv) {
      // Clean up any object URLs from file blocks
      conv.blocks.forEach(block => {
        if (block.type === 'file' && block.previewUrl) {
          URL.revokeObjectURL(block.previewUrl);
        }
      });
    }
    conversations.value.delete(peerId);
    if (activeConversationId.value === peerId) {
      activeConversationId.value = null;
    }
  }

  function addPendingMessage(message: P2PMessage) {
    pendingMessages.value.set(message.id, message);
  }

  function removePendingMessage(messageId: string) {
    pendingMessages.value.delete(messageId);
  }

  return {
    // State
    conversations,
    activeConversationId,
    typingPeers,
    pendingMessages,
    // Getters
    activeConversation,
    conversationList,
    totalUnread,
    // Actions
    createConversation,
    setActiveConversation,
    addBlock,
    updateBlock,
    addWidgetResponse,
    setTypingIndicator,
    clearConversation,
    removeConversation,
    addPendingMessage,
    removePendingMessage,
  };
});
