<script setup lang="ts">
import { computed } from 'vue';
import { Copy, Check } from 'lucide-vue-next';
import { ref } from 'vue';
import type { TextBlock } from '@/types/blocks';

const props = defineProps<{
  block: TextBlock;
}>();

const copied = ref(false);

const formattedTime = computed(() => {
  return new Date(props.block.timestamp).toLocaleTimeString([], { 
    hour: '2-digit', 
    minute: '2-digit' 
  });
});

// Simple markdown-like formatting
const formattedContent = computed(() => {
  if (props.block.format !== 'markdown') {
    return props.block.content;
  }
  
  let text = props.block.content;
  // Bold
  text = text.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
  // Italic
  text = text.replace(/\*(.*?)\*/g, '<em>$1</em>');
  // Inline code
  text = text.replace(/`(.*?)`/g, '<code class="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-sm">$1</code>');
  // Links
  text = text.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-blue-500 hover:underline" target="_blank">$1</a>');
  
  return text;
});

async function copyContent() {
  await navigator.clipboard.writeText(props.block.content);
  copied.value = true;
  setTimeout(() => copied.value = false, 2000);
}
</script>

<template>
  <div 
    class="text-block group relative"
    :class="block.isLocal ? 'ml-auto' : 'mr-auto'"
  >
    <div 
      class="max-w-md px-4 py-2 rounded-2xl"
      :class="block.isLocal 
        ? 'bg-blue-500 text-white rounded-br-md' 
        : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white rounded-bl-md'"
    >
      <p 
        v-if="block.format === 'markdown'" 
        v-html="formattedContent"
        class="whitespace-pre-wrap break-words"
      />
      <p v-else class="whitespace-pre-wrap break-words">
        {{ block.content }}
      </p>
    </div>
    
    <!-- Metadata & Actions -->
    <div 
      class="flex items-center gap-2 mt-1 opacity-0 group-hover:opacity-100 transition-opacity"
      :class="block.isLocal ? 'justify-end' : 'justify-start'"
    >
      <span class="text-xs text-gray-500">{{ formattedTime }}</span>
      <button 
        @click="copyContent"
        class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
        title="Copy"
      >
        <Check v-if="copied" class="w-3 h-3 text-green-500" />
        <Copy v-else class="w-3 h-3" />
      </button>
    </div>
  </div>
</template>
