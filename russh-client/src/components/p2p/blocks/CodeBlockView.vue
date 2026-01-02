<script setup lang="ts">
import { ref, computed } from 'vue';
import { Copy, Check, ChevronDown, ChevronUp, FileCode } from 'lucide-vue-next';
import type { CodeBlock } from '@/types/blocks';

const props = defineProps<{
  block: CodeBlock;
}>();

const copied = ref(false);
const isExpanded = ref(false);

const lines = computed(() => props.block.content.split('\n'));
const isCollapsible = computed(() => lines.value.length > 20);
const displayedContent = computed(() => {
  if (!isCollapsible.value || isExpanded.value) {
    return props.block.content;
  }
  return lines.value.slice(0, 20).join('\n');
});

const languageColors: Record<string, string> = {
  javascript: 'bg-yellow-500',
  typescript: 'bg-blue-500',
  python: 'bg-green-500',
  rust: 'bg-orange-500',
  go: 'bg-cyan-500',
  java: 'bg-red-500',
  cpp: 'bg-purple-500',
  c: 'bg-gray-500',
  html: 'bg-orange-400',
  css: 'bg-blue-400',
  json: 'bg-gray-400',
  yaml: 'bg-pink-500',
  bash: 'bg-green-600',
  sql: 'bg-indigo-500',
};

const languageColor = computed(() => {
  return languageColors[props.block.language.toLowerCase()] || 'bg-gray-500';
});

const formattedTime = computed(() => {
  return new Date(props.block.timestamp).toLocaleTimeString([], { 
    hour: '2-digit', 
    minute: '2-digit' 
  });
});

async function copyCode() {
  await navigator.clipboard.writeText(props.block.content);
  copied.value = true;
  setTimeout(() => copied.value = false, 2000);
}
</script>

<template>
  <div 
    class="code-block w-full max-w-2xl"
    :class="block.isLocal ? 'ml-auto' : 'mr-auto'"
  >
    <div class="bg-gray-900 rounded-lg overflow-hidden border border-gray-700">
      <!-- Header -->
      <div class="flex items-center justify-between px-3 py-2 bg-gray-800 border-b border-gray-700">
        <div class="flex items-center gap-2">
          <FileCode class="w-4 h-4 text-gray-400" />
          <span 
            class="px-2 py-0.5 text-xs font-medium text-white rounded"
            :class="languageColor"
          >
            {{ block.language }}
          </span>
          <span v-if="block.filename" class="text-xs text-gray-400">
            {{ block.filename }}
          </span>
        </div>
        <button 
          @click="copyCode"
          class="flex items-center gap-1 px-2 py-1 text-xs text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors"
        >
          <Check v-if="copied" class="w-3 h-3 text-green-500" />
          <Copy v-else class="w-3 h-3" />
          {{ copied ? 'Copied!' : 'Copy' }}
        </button>
      </div>
      
      <!-- Code Content -->
      <div class="relative">
        <pre class="p-4 overflow-x-auto text-sm"><code 
          class="text-gray-100 font-mono"
          :class="block.showLineNumbers ? 'block' : ''"
        ><template v-if="block.showLineNumbers"><span 
          v-for="(line, i) in displayedContent.split('\n')" 
          :key="i"
          class="block"
><span class="inline-block w-8 text-gray-500 text-right mr-4 select-none">{{ i + 1 }}</span>{{ line }}</span></template><template v-else>{{ displayedContent }}</template></code></pre>
        
        <!-- Expand/Collapse -->
        <div 
          v-if="isCollapsible"
          class="absolute bottom-0 left-0 right-0 flex justify-center pb-2 pt-8 bg-gradient-to-t from-gray-900 to-transparent"
        >
          <button 
            @click="isExpanded = !isExpanded"
            class="flex items-center gap-1 px-3 py-1 text-xs text-gray-400 hover:text-white bg-gray-800 hover:bg-gray-700 rounded-full transition-colors"
          >
            <ChevronUp v-if="isExpanded" class="w-3 h-3" />
            <ChevronDown v-else class="w-3 h-3" />
            {{ isExpanded ? 'Collapse' : `Show ${lines.length - 20} more lines` }}
          </button>
        </div>
      </div>
    </div>
    
    <!-- Timestamp -->
    <div class="mt-1 text-xs text-gray-500" :class="block.isLocal ? 'text-right' : 'text-left'">
      {{ formattedTime }}
    </div>
  </div>
</template>
