<script setup lang="ts">
import { computed } from 'vue';

import type { ConnectionStatus } from '@/types/ssh';

const props = defineProps<{
  state?: ConnectionStatus;
  showLabel?: boolean;
}>();

const statusConfig = computed(() => {
  switch (props.state) {
    case 'connected':
      return { color: 'bg-green-500', label: 'Connected', pulse: false };
    case 'connecting':
      return { color: 'bg-yellow-500', label: 'Connecting', pulse: true };
    case 'reconnecting':
      return { color: 'bg-orange-500', label: 'Reconnecting', pulse: true };
    case 'error':
      return { color: 'bg-red-500', label: 'Error', pulse: false };
    default:
      return { color: 'bg-gray-400', label: 'Disconnected', pulse: false };
  }
});
</script>

<template>
  <div class="flex items-center gap-2">
    <span 
      class="relative flex h-2.5 w-2.5"
      :title="statusConfig.label"
    >
      <span 
        v-if="statusConfig.pulse"
        class="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75"
        :class="statusConfig.color"
      />
      <span 
        class="relative inline-flex rounded-full h-2.5 w-2.5"
        :class="statusConfig.color"
      />
    </span>
    <span 
      v-if="showLabel" 
      class="text-sm text-gray-600 dark:text-gray-400"
    >
      {{ statusConfig.label }}
    </span>
  </div>
</template>
