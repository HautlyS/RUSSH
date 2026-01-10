<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { Server, MoreVertical } from 'lucide-vue-next';
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
    router.push(`/terminal/${connection.value?.sessionId}`);
  } else {
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
}
</script>

<template>
  <div 
    @click="handleClick"
    @contextmenu="handleContextMenu"
    :class="[
      'group flex items-center gap-3 px-3 py-2 rounded-xl cursor-pointer transition-all',
      isConnected 
        ? 'bg-green-500/10 hover:bg-green-500/15 border border-green-500/20' 
        : 'hover:bg-white/5 border border-transparent'
    ]"
  >
    <!-- Icon with status -->
    <div class="relative flex-shrink-0">
      <div 
        class="w-8 h-8 rounded-lg flex items-center justify-center"
        :style="{ backgroundColor: (profile.color || '#6b7280') + '20' }"
      >
        <Server 
          class="w-4 h-4" 
          :style="{ color: profile.color || '#6b7280' }"
        />
      </div>
      <!-- Status dot -->
      <div 
        :class="[
          'absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border-2 border-gray-950',
          isConnected ? 'bg-green-500' : isConnecting ? 'bg-yellow-500 animate-pulse' : 'bg-gray-600'
        ]"
      />
    </div>
    
    <!-- Content -->
    <div v-if="!collapsed" class="flex-1 min-w-0">
      <div 
        :class="[
          'text-sm font-medium truncate transition-colors',
          isConnected ? 'text-green-400' : 'text-gray-300 group-hover:text-white'
        ]"
      >
        {{ profile.name }}
      </div>
      <div class="text-xs text-gray-500 truncate">
        {{ profile.username }}@{{ profile.host }}
      </div>
    </div>
    
    <!-- Actions -->
    <button 
      v-if="!collapsed"
      @click.stop
      class="p-1.5 opacity-0 group-hover:opacity-100 hover:bg-white/10 rounded-lg transition-all"
    >
      <MoreVertical class="w-4 h-4 text-gray-400" />
    </button>
  </div>
</template>
