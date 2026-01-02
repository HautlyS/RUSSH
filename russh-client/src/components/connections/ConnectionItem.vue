<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { Server, Wifi, WifiOff, MoreVertical } from 'lucide-vue-next';
import type { ConnectionProfile } from '@/types/ssh';

const props = defineProps<{
  profile: ConnectionProfile;
  collapsed?: boolean;
}>();

const router = useRouter();
const connectionStore = useConnectionStore();

const connection = computed(() => connectionStore.getConnection(props.profile.id));
const isConnected = computed(() => connection.value?.status === 'connected');
const isConnecting = computed(() => connection.value?.status === 'connecting');

async function handleClick() {
  if (isConnected.value) {
    // Navigate to terminal
    router.push(`/terminal/${connection.value?.sessionId}`);
  } else {
    // Connect
    try {
      const sessionId = await connectionStore.connect(props.profile.id);
      router.push(`/terminal/${sessionId}`);
    } catch (e) {
      // Error handled by store
    }
  }
}

function handleContextMenu(event: MouseEvent) {
  event.preventDefault();
  // TODO: Show context menu
}
</script>

<template>
  <div 
    @click="handleClick"
    @contextmenu="handleContextMenu"
    :class="[
      'group flex items-center gap-2 px-2 py-1.5 rounded-lg cursor-pointer transition-colors',
      isConnected 
        ? 'bg-green-50 dark:bg-green-900/20 hover:bg-green-100 dark:hover:bg-green-900/30' 
        : 'hover:bg-gray-100 dark:hover:bg-gray-800'
    ]"
  >
    <!-- Status indicator -->
    <div 
      :class="[
        'w-2 h-2 rounded-full flex-shrink-0',
        isConnected ? 'bg-green-500' : isConnecting ? 'bg-yellow-500 animate-pulse' : 'bg-gray-300 dark:bg-gray-600'
      ]"
    />
    
    <!-- Color indicator -->
    <div 
      v-if="profile.color && !collapsed"
      class="w-1 h-6 rounded-full flex-shrink-0"
      :style="{ backgroundColor: profile.color }"
    />
    
    <!-- Icon -->
    <Server v-if="collapsed" class="w-4 h-4 text-gray-500" />
    
    <!-- Content -->
    <div v-if="!collapsed" class="flex-1 min-w-0">
      <div class="text-sm font-medium truncate">{{ profile.name }}</div>
      <div class="text-xs text-gray-500 truncate">{{ profile.username }}@{{ profile.host }}</div>
    </div>
    
    <!-- Actions -->
    <button 
      v-if="!collapsed"
      @click.stop
      class="p-1 opacity-0 group-hover:opacity-100 hover:bg-gray-200 dark:hover:bg-gray-700 rounded transition-opacity"
    >
      <MoreVertical class="w-4 h-4" />
    </button>
  </div>
</template>
