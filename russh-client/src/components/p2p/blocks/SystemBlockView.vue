<script setup lang="ts">
import { computed } from 'vue';
import { Info, AlertTriangle, XCircle, CheckCircle } from 'lucide-vue-next';
import type { SystemBlock } from '@/types/blocks';

const props = defineProps<{
  block: SystemBlock;
}>();

const icon = computed(() => {
  switch (props.block.level) {
    case 'success': return CheckCircle;
    case 'warning': return AlertTriangle;
    case 'error': return XCircle;
    default: return Info;
  }
});

const colors = computed(() => {
  switch (props.block.level) {
    case 'success': return 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 border-green-200 dark:border-green-800';
    case 'warning': return 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 border-yellow-200 dark:border-yellow-800';
    case 'error': return 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 border-red-200 dark:border-red-800';
    default: return 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 border-blue-200 dark:border-blue-800';
  }
});

const formattedTime = computed(() => {
  return new Date(props.block.timestamp).toLocaleTimeString([], { 
    hour: '2-digit', 
    minute: '2-digit' 
  });
});
</script>

<template>
  <div class="system-block w-full flex justify-center my-2">
    <div 
      class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-xs border"
      :class="colors"
    >
      <component :is="icon" class="w-3.5 h-3.5" />
      <span>{{ block.message }}</span>
      <span class="opacity-60">{{ formattedTime }}</span>
    </div>
  </div>
</template>
