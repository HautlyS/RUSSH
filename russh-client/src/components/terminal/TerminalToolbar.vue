<script setup lang="ts">
import { ref, computed } from 'vue';
import { Trash2, Search, Settings, Columns, Copy, Download, Maximize2, Terminal } from 'lucide-vue-next';
import { useVisualEffects } from '@/composables/useVisualEffects';
import DecryptedText from '@/components/extra/DecryptedText.vue';

const props = defineProps<{
  hostname?: string;
  connected?: boolean;
}>();

const emit = defineEmits<{
  (e: 'clear'): void;
  (e: 'search'): void;
  (e: 'settings'): void;
  (e: 'split'): void;
  (e: 'copy'): void;
  (e: 'export'): void;
  (e: 'fullscreen'): void;
}>();

const { isDecryptedTextEnabled, visualEffects } = useVisualEffects();

const showSearch = ref(false);
const searchQuery = ref('');

const displayHostname = computed(() => props.hostname || 'Terminal');
</script>

<template>
  <div class="terminal-toolbar flex items-center gap-1 px-2 py-1 bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
    <!-- Connection Status -->
    <div class="flex items-center gap-2 px-2 text-sm text-gray-600 dark:text-gray-400">
      <Terminal class="w-4 h-4" />
      <DecryptedText
        v-if="isDecryptedTextEnabled && connected"
        :text="displayHostname"
        :speed="30"
        :max-iterations="8"
        animate-on="view"
        characters="01"
        class-name="text-green-500 font-mono"
        encrypted-class-name="text-gray-500 font-mono"
      />
      <span v-else class="font-mono">{{ displayHostname }}</span>
    </div>

    <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-1" />

    <button
      @click="emit('clear')"
      class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
      title="Clear terminal (Ctrl+L)"
    >
      <Trash2 class="w-4 h-4" />
    </button>
    
    <button
      @click="showSearch = !showSearch; emit('search')"
      class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
      :class="{ 'bg-blue-100 dark:bg-blue-900/30 text-blue-600': showSearch }"
      title="Search (Ctrl+F)"
    >
      <Search class="w-4 h-4" />
    </button>
    
    <button
      @click="emit('copy')"
      class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
      title="Copy selection (Ctrl+Shift+C)"
    >
      <Copy class="w-4 h-4" />
    </button>
    
    <div class="flex-1" />
    
    <button
      @click="emit('split')"
      class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
      title="Split terminal"
    >
      <Columns class="w-4 h-4" />
    </button>
    
    <button
      @click="emit('export')"
      class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
      title="Export output"
    >
      <Download class="w-4 h-4" />
    </button>
    
    <button
      @click="emit('fullscreen')"
      class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
      title="Fullscreen (F11)"
    >
      <Maximize2 class="w-4 h-4" />
    </button>
    
    <button
      @click="emit('settings')"
      class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
      title="Terminal settings"
    >
      <Settings class="w-4 h-4" />
    </button>
  </div>
</template>
