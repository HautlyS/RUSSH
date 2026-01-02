<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { usePlatform } from '@/composables/usePlatform';
import { Server, Wifi, WifiOff, ChevronRight, Terminal, FolderOpen, Trash2 } from 'lucide-vue-next';
import type { ConnectionProfile } from '@/types/ssh';

const props = defineProps<{
  profile: ConnectionProfile;
}>();

const emit = defineEmits<{
  connect: [id: string];
  disconnect: [id: string];
  delete: [id: string];
}>();

const router = useRouter();
const connectionStore = useConnectionStore();
const { hapticFeedback } = usePlatform();

// Connection state
const connectionState = computed(() => 
  connectionStore.getConnection(props.profile.id)
);

const isConnected = computed(() => 
  connectionState.value?.status === 'connected'
);

const isConnecting = computed(() => 
  connectionState.value?.status === 'connecting'
);

// Swipe state
let touchStartX = 0;
let touchCurrentX = 0;
let isSwiping = false;

function onTouchStart(e: TouchEvent) {
  touchStartX = e.touches[0].clientX;
  isSwiping = false;
}

function onTouchMove(e: TouchEvent) {
  touchCurrentX = e.touches[0].clientX;
  const diff = touchStartX - touchCurrentX;
  if (Math.abs(diff) > 20) {
    isSwiping = true;
  }
}

function onTouchEnd() {
  if (isSwiping) {
    const diff = touchStartX - touchCurrentX;
    if (diff > 80) {
      // Swiped left - show delete
      hapticFeedback('warning');
      if (confirm(`Delete "${props.profile.name}"?`)) {
        emit('delete', props.profile.id);
      }
    }
  }
  isSwiping = false;
}

// Actions
function handleTap() {
  if (isSwiping) return;
  hapticFeedback('light');
  
  if (isConnected.value) {
    // Go to terminal
    router.push(`/terminal/${connectionState.value?.sessionId}`);
  } else {
    emit('connect', props.profile.id);
  }
}

function openTerminal() {
  hapticFeedback('light');
  if (connectionState.value?.sessionId) {
    router.push(`/terminal/${connectionState.value.sessionId}`);
  }
}

function openFiles() {
  hapticFeedback('light');
  if (connectionState.value?.sessionId) {
    router.push(`/files/${connectionState.value.sessionId}`);
  }
}

function disconnect() {
  hapticFeedback('medium');
  emit('disconnect', props.profile.id);
}
</script>

<template>
  <div
    class="mobile-connection-card bg-white dark:bg-gray-800 rounded-xl shadow-sm overflow-hidden"
    @touchstart="onTouchStart"
    @touchmove="onTouchMove"
    @touchend="onTouchEnd"
  >
    <!-- Main Card Content -->
    <button
      @click="handleTap"
      class="w-full flex items-center gap-4 p-4 text-left"
    >
      <!-- Status Icon -->
      <div 
        class="w-12 h-12 rounded-full flex items-center justify-center flex-shrink-0"
        :class="[
          isConnected ? 'bg-green-100 dark:bg-green-900/30' : 'bg-gray-100 dark:bg-gray-700',
          isConnecting ? 'animate-pulse' : ''
        ]"
      >
        <Server 
          class="w-6 h-6"
          :class="isConnected ? 'text-green-600' : 'text-gray-500'"
        />
      </div>
      
      <!-- Info -->
      <div class="flex-1 min-w-0">
        <div class="font-semibold text-gray-900 dark:text-white truncate">
          {{ profile.name }}
        </div>
        <div class="text-sm text-gray-500 dark:text-gray-400 truncate">
          {{ profile.username }}@{{ profile.host }}:{{ profile.port }}
        </div>
        <div class="flex items-center gap-1 mt-1">
          <component 
            :is="isConnected ? Wifi : WifiOff" 
            class="w-3 h-3"
            :class="isConnected ? 'text-green-500' : 'text-gray-400'"
          />
          <span class="text-xs" :class="isConnected ? 'text-green-500' : 'text-gray-400'">
            {{ isConnecting ? 'Connecting...' : isConnected ? 'Connected' : 'Disconnected' }}
          </span>
        </div>
      </div>
      
      <!-- Arrow -->
      <ChevronRight class="w-5 h-5 text-gray-400 flex-shrink-0" />
    </button>
    
    <!-- Quick Actions (when connected) -->
    <div 
      v-if="isConnected"
      class="flex items-center border-t border-gray-100 dark:border-gray-700"
    >
      <button
        @click="openTerminal"
        class="flex-1 flex items-center justify-center gap-2 py-3 text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
      >
        <Terminal class="w-4 h-4" />
        Terminal
      </button>
      
      <div class="w-px h-8 bg-gray-100 dark:bg-gray-700" />
      
      <button
        @click="openFiles"
        class="flex-1 flex items-center justify-center gap-2 py-3 text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
      >
        <FolderOpen class="w-4 h-4" />
        Files
      </button>
      
      <div class="w-px h-8 bg-gray-100 dark:bg-gray-700" />
      
      <button
        @click="disconnect"
        class="flex-1 flex items-center justify-center gap-2 py-3 text-sm text-red-500 hover:bg-gray-50 dark:hover:bg-gray-700"
      >
        <WifiOff class="w-4 h-4" />
        Disconnect
      </button>
    </div>
  </div>
</template>
