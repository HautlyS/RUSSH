<script setup lang="ts">
import { X, Circle } from 'lucide-vue-next';

const props = defineProps<{
  id: string;
  title: string;
  active?: boolean;
  hasActivity?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select'): void;
  (e: 'close'): void;
}>();
</script>

<template>
  <div
    class="terminal-tab flex items-center gap-2 px-3 py-2 cursor-pointer border-b-2 transition-colors"
    :class="[
      active 
        ? 'border-blue-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-white' 
        : 'border-transparent hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400'
    ]"
    @click="emit('select')"
  >
    <Circle 
      v-if="hasActivity && !active" 
      class="w-2 h-2 text-blue-500 fill-current" 
    />
    <span class="text-sm truncate max-w-[120px]">{{ title }}</span>
    <button
      @click.stop="emit('close')"
      class="p-0.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
      title="Close tab"
    >
      <X class="w-3.5 h-3.5" />
    </button>
  </div>
</template>
