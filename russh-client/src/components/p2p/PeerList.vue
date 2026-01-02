<script setup lang="ts">
import { Monitor, Smartphone, Tablet, X } from 'lucide-vue-next';
import ConnectionQuality from './ConnectionQuality.vue';
import type { P2PPeer } from '@/types/p2p';

defineProps<{
  peers: P2PPeer[];
}>();

const emit = defineEmits<{
  (e: 'disconnect', peerId: string): void;
  (e: 'select', peer: P2PPeer): void;
}>();

function getDeviceIcon(deviceType?: string) {
  switch (deviceType) {
    case 'mobile': return Smartphone;
    case 'tablet': return Tablet;
    default: return Monitor;
  }
}
</script>

<template>
  <div class="peer-list">
    <div v-if="peers.length === 0" class="p-4 text-center text-gray-500 dark:text-gray-400">
      <p>No connected peers</p>
      <p class="text-sm mt-1">Share your Node ID or scan a QR code to connect</p>
    </div>
    
    <div
      v-for="peer in peers"
      :key="peer.id"
      class="flex items-center gap-3 p-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 cursor-pointer border-b border-gray-100 dark:border-gray-700 last:border-0"
      @click="emit('select', peer)"
    >
      <div class="p-2 bg-blue-100 dark:bg-blue-900/30 rounded-lg">
        <component :is="getDeviceIcon(peer.deviceType)" class="w-5 h-5 text-blue-600 dark:text-blue-400" />
      </div>
      
      <div class="flex-1 min-w-0">
        <div class="font-medium text-gray-900 dark:text-white truncate">
          {{ peer.name || 'Unknown Device' }}
        </div>
        <div class="text-xs text-gray-500 dark:text-gray-400 font-mono truncate">
          {{ peer.id.slice(0, 16) }}...
        </div>
      </div>
      
      <ConnectionQuality :latency="peer.latency" :connection-type="peer.connectionType" />
      
      <button
        @click.stop="emit('disconnect', peer.id)"
        class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-400 hover:text-red-500 transition-colors"
        title="Disconnect"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>
