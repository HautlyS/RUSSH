<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue';
import { MessageSquare, Users, Wifi, WifiOff, Loader2 } from 'lucide-vue-next';
import { useP2P } from '@/composables/useP2P';
import { useP2PMessaging } from '@/composables/useP2PMessaging';
import { useP2PTerminalStore } from '@/stores/p2pTerminal';
import { useVisualEffects } from '@/composables/useVisualEffects';
import BlockComposer from './BlockComposer.vue';
import TextBlockView from './blocks/TextBlockView.vue';
import CodeBlockView from './blocks/CodeBlockView.vue';
import FileBlockView from './blocks/FileBlockView.vue';
import WidgetBlockView from './blocks/WidgetBlockView.vue';
import SystemBlockView from './blocks/SystemBlockView.vue';
import Lightning from '@/components/extra/Lightning.vue';
import type { WidgetConfig, WidgetType, FileBlock } from '@/types/blocks';
import type { P2PPeer } from '@/types/p2p';

const { nodeInfo, peers, isOnline, initialize: initP2P } = useP2P();
const { 
  localNodeId,
  initialize: initMessaging,
  sendTextBlock,
  sendCodeBlock,
  sendFileBlock,
  sendWidgetBlock,
  sendSystemBlock,
  sendTypingIndicator,
  sendWidgetResponse,
} = useP2PMessaging();
const terminalStore = useP2PTerminalStore();
const { isLightningEnabledFor, visualEffects } = useVisualEffects();

const isLightningEnabled = isLightningEnabledFor('p2p');
const messagesRef = ref<HTMLDivElement>();
const isInitializing = ref(true);

const activeConversation = computed(() => terminalStore.activeConversation);
const activePeer = computed(() => {
  if (!terminalStore.activeConversationId) return null;
  return peers.value.find(p => p.id === terminalStore.activeConversationId) || null;
});

// Initialize
onMounted(async () => {
  await initP2P();
  if (nodeInfo.value) {
    await initMessaging(nodeInfo.value.nodeId);
  }
  isInitializing.value = false;
});

// Auto-scroll to bottom on new messages
watch(
  () => activeConversation.value?.blocks.length,
  async () => {
    await nextTick();
    scrollToBottom();
  }
);

function scrollToBottom() {
  if (messagesRef.value) {
    messagesRef.value.scrollTop = messagesRef.value.scrollHeight;
  }
}

function selectPeer(peer: P2PPeer) {
  terminalStore.createConversation(peer);
  terminalStore.setActiveConversation(peer.id);
  sendSystemBlock(peer.id, `Connected to ${peer.name || peer.id.slice(0, 8)}`, 'success');
}

function handleSendText(content: string, format: 'plain' | 'markdown') {
  if (!terminalStore.activeConversationId) return;
  sendTextBlock(terminalStore.activeConversationId, content, format);
}

function handleSendCode(content: string, language: string, filename?: string) {
  if (!terminalStore.activeConversationId) return;
  sendCodeBlock(terminalStore.activeConversationId, content, language, filename);
}

function handleSendFile(file: File) {
  if (!terminalStore.activeConversationId) return;
  sendFileBlock(terminalStore.activeConversationId, file);
}

function handleSendWidget(widgetType: WidgetType, config: WidgetConfig) {
  if (!terminalStore.activeConversationId) return;
  sendWidgetBlock(terminalStore.activeConversationId, widgetType, config);
}

function handleTyping(isTyping: boolean) {
  if (!terminalStore.activeConversationId) return;
  sendTypingIndicator(terminalStore.activeConversationId, isTyping);
}

function handleWidgetResponse(blockId: string, value: unknown) {
  if (!terminalStore.activeConversationId) return;
  sendWidgetResponse(terminalStore.activeConversationId, blockId, value);
}

function handleFileDownload(block: FileBlock) {
  // Trigger download
  if (block.data) {
    const blob = new Blob([block.data], { type: block.mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = block.filename;
    a.click();
    URL.revokeObjectURL(url);
  }
}
</script>

<template>
  <div class="p2p-terminal h-full flex relative">
    <!-- Lightning Background -->
    <div 
      v-if="isLightningEnabled" 
      class="absolute inset-0 z-0 opacity-15"
    >
      <Lightning
        :hue="200"
        :intensity="visualEffects.lightning.intensity"
        :speed="visualEffects.lightning.speed"
        :size="1"
      />
    </div>

    <!-- Sidebar - Peer List -->
    <div class="relative z-10 w-72 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex flex-col">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between">
          <h2 class="font-semibold text-gray-900 dark:text-white flex items-center gap-2">
            <Users class="w-5 h-5" />
            Peers
          </h2>
          <div class="flex items-center gap-1">
            <Wifi v-if="isOnline" class="w-4 h-4 text-green-500" />
            <WifiOff v-else class="w-4 h-4 text-red-500" />
          </div>
        </div>
      </div>
      
      <!-- Loading State -->
      <div v-if="isInitializing" class="flex-1 flex items-center justify-center">
        <Loader2 class="w-6 h-6 text-gray-400 animate-spin" />
      </div>
      
      <!-- Peer List -->
      <div v-else class="flex-1 overflow-y-auto">
        <div v-if="peers.length === 0" class="p-4 text-center text-gray-500 text-sm">
          No peers connected
        </div>
        <button 
          v-for="peer in peers"
          :key="peer.id"
          @click="selectPeer(peer)"
          class="w-full p-4 text-left hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors border-b border-gray-100 dark:border-gray-700"
          :class="{ 'bg-blue-50 dark:bg-blue-900/30': terminalStore.activeConversationId === peer.id }"
        >
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-full bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center text-white font-medium">
              {{ (peer.name || peer.id)[0].toUpperCase() }}
            </div>
            <div class="flex-1 min-w-0">
              <p class="font-medium text-gray-900 dark:text-white truncate">
                {{ peer.name || `Peer ${peer.id.slice(0, 8)}` }}
              </p>
              <p class="text-xs text-gray-500 truncate">
                {{ peer.connectionType }} • {{ peer.latencyMs || '?' }}ms
              </p>
            </div>
            <div 
              v-if="terminalStore.conversations.get(peer.id)?.unreadCount"
              class="w-5 h-5 rounded-full bg-blue-500 text-white text-xs flex items-center justify-center"
            >
              {{ terminalStore.conversations.get(peer.id)?.unreadCount }}
            </div>
          </div>
        </button>
      </div>
    </div>
    
    <!-- Main Chat Area -->
    <div class="relative z-10 flex-1 flex flex-col bg-gray-50 dark:bg-gray-900">
      <!-- No Conversation Selected -->
      <div 
        v-if="!activeConversation"
        class="flex-1 flex flex-col items-center justify-center text-gray-500"
      >
        <MessageSquare class="w-16 h-16 mb-4 opacity-50" />
        <p class="text-lg font-medium">Select a peer to start chatting</p>
        <p class="text-sm mt-1">Choose from the connected peers on the left</p>
      </div>
      
      <!-- Active Conversation -->
      <template v-else>
        <!-- Header -->
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-full bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center text-white font-medium">
              {{ (activePeer?.name || activePeer?.id || '?')[0].toUpperCase() }}
            </div>
            <div>
              <h3 class="font-semibold text-gray-900 dark:text-white">
                {{ activePeer?.name || `Peer ${activePeer?.id?.slice(0, 8)}` }}
              </h3>
              <p class="text-xs text-gray-500">
                {{ activePeer?.connectionType }} connection
                <span v-if="activeConversation.isTyping" class="ml-2 text-blue-500">
                  typing...
                </span>
              </p>
            </div>
          </div>
        </div>
        
        <!-- Messages -->
        <div 
          ref="messagesRef"
          class="flex-1 overflow-y-auto p-6 space-y-4"
        >
          <TransitionGroup name="block">
            <template v-for="block in activeConversation.blocks" :key="block.id">
              <TextBlockView 
                v-if="block.type === 'text'" 
                :block="block" 
              />
              <CodeBlockView 
                v-else-if="block.type === 'code'" 
                :block="block" 
              />
              <FileBlockView 
                v-else-if="block.type === 'file'" 
                :block="block"
                @download="handleFileDownload"
              />
              <WidgetBlockView 
                v-else-if="block.type === 'widget'" 
                :block="block"
                :local-node-id="localNodeId"
                @respond="handleWidgetResponse"
              />
              <SystemBlockView 
                v-else-if="block.type === 'system'" 
                :block="block" 
              />
            </template>
          </TransitionGroup>
        </div>
        
        <!-- Composer -->
        <BlockComposer
          @send-text="handleSendText"
          @send-code="handleSendCode"
          @send-file="handleSendFile"
          @send-widget="handleSendWidget"
          @typing="handleTyping"
        />
      </template>
    </div>
  </div>
</template>

<style scoped>
.block-enter-active {
  transition: all 0.3s ease-out;
}
.block-leave-active {
  transition: all 0.2s ease-in;
}
.block-enter-from {
  opacity: 0;
  transform: translateY(20px);
}
.block-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}
</style>
