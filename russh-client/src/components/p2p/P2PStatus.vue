<script setup lang="ts">
import { computed } from 'vue';
import { Wifi, WifiOff, Copy, Check } from 'lucide-vue-next';
import { ref } from 'vue';

const props = defineProps<{
  isOnline: boolean;
  nodeId?: string;
  peerCount?: number;
}>();

const copied = ref(false);

async function copyNodeId() {
  if (props.nodeId) {
    await navigator.clipboard.writeText(props.nodeId);
    copied.value = true;
    setTimeout(() => copied.value = false, 2000);
  }
}

const truncatedNodeId = computed(() => {
  if (!props.nodeId) return '';
  return `${props.nodeId.slice(0, 8)}...${props.nodeId.slice(-8)}`;
});
</script>

<template>
  <div class="p2p-status p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
    <div class="flex items-center gap-3">
      <div 
        class="p-2 rounded-full"
        :class="isOnline ? 'bg-green-100 dark:bg-green-900/30' : 'bg-gray-100 dark:bg-gray-700'"
      >
        <Wifi v-if="isOnline" class="w-5 h-5 text-green-600 dark:text-green-400" />
        <WifiOff v-else class="w-5 h-5 text-gray-400" />
      </div>
      
      <div class="flex-1">
        <div class="flex items-center gap-2">
          <span class="font-medium text-gray-900 dark:text-white">
            {{ isOnline ? 'Online' : 'Offline' }}
          </span>
          <span v-if="peerCount" class="text-sm text-gray-500 dark:text-gray-400">
            ({{ peerCount }} peer{{ peerCount !== 1 ? 's' : '' }})
          </span>
        </div>
        
        <div v-if="nodeId" class="flex items-center gap-2 mt-1">
          <code class="text-xs text-gray-500 dark:text-gray-400 font-mono">
            {{ truncatedNodeId }}
          </code>
          <button
            @click="copyNodeId"
            class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
            :title="copied ? 'Copied!' : 'Copy Node ID'"
          >
            <Check v-if="copied" class="w-3.5 h-3.5 text-green-500" />
            <Copy v-else class="w-3.5 h-3.5 text-gray-400" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
