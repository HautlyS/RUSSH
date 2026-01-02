<script setup lang="ts">
import { ref, computed } from 'vue';
import { Check, X, Send } from 'lucide-vue-next';
import type { WidgetBlock, WidgetResponse } from '@/types/blocks';

const props = defineProps<{
  block: WidgetBlock;
  localNodeId: string;
}>();

const emit = defineEmits<{
  respond: [blockId: string, value: unknown];
}>();

const inputValue = ref('');
const selectedOptions = ref<number[]>([]);

const formattedTime = computed(() => {
  return new Date(props.block.timestamp).toLocaleTimeString([], { 
    hour: '2-digit', 
    minute: '2-digit' 
  });
});

const hasResponded = computed(() => {
  return props.block.responses?.some(r => r.responderId === props.localNodeId);
});

const pollResults = computed(() => {
  if (props.block.config.type !== 'poll') return [];
  
  const votes = new Map<number, number>();
  props.block.responses?.forEach(r => {
    const indices = r.value as number[];
    indices.forEach(i => votes.set(i, (votes.get(i) || 0) + 1));
  });
  
  const total = props.block.responses?.length || 0;
  return props.block.config.options.map((opt, i) => ({
    option: opt,
    count: votes.get(i) || 0,
    percentage: total > 0 ? ((votes.get(i) || 0) / total) * 100 : 0,
  }));
});

function handleButtonClick(action: string) {
  emit('respond', props.block.id, { action });
}

function handleInputSubmit() {
  if (inputValue.value.trim()) {
    emit('respond', props.block.id, inputValue.value);
    inputValue.value = '';
  }
}

function handlePollVote() {
  if (selectedOptions.value.length > 0) {
    emit('respond', props.block.id, selectedOptions.value);
  }
}

function togglePollOption(index: number) {
  if (props.block.config.type !== 'poll') return;
  
  if (props.block.config.allowMultiple) {
    const idx = selectedOptions.value.indexOf(index);
    if (idx > -1) {
      selectedOptions.value.splice(idx, 1);
    } else {
      selectedOptions.value.push(index);
    }
  } else {
    selectedOptions.value = [index];
  }
}

function handleConfirm(confirmed: boolean) {
  emit('respond', props.block.id, confirmed);
}
</script>

<template>
  <div 
    class="widget-block w-full max-w-sm"
    :class="block.isLocal ? 'ml-auto' : 'mr-auto'"
  >
    <div 
      class="rounded-lg overflow-hidden border p-4"
      :class="block.isLocal 
        ? 'bg-purple-50 dark:bg-purple-900/20 border-purple-200 dark:border-purple-800' 
        : 'bg-gray-100 dark:bg-gray-800 border-gray-200 dark:border-gray-700'"
    >
      <!-- Button Widget -->
      <template v-if="block.config.type === 'button'">
        <button 
          @click="handleButtonClick(block.config.action)"
          :disabled="hasResponded"
          class="w-full px-4 py-2 rounded-lg font-medium transition-colors disabled:opacity-50"
          :class="{
            'bg-blue-500 hover:bg-blue-600 text-white': block.config.variant === 'primary',
            'bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white': block.config.variant === 'secondary',
            'bg-red-500 hover:bg-red-600 text-white': block.config.variant === 'danger',
          }"
        >
          {{ block.config.label }}
        </button>
        <p v-if="block.responses?.length" class="mt-2 text-xs text-gray-500 text-center">
          {{ block.responses.length }} response(s)
        </p>
      </template>
      
      <!-- Input Widget -->
      <template v-else-if="block.config.type === 'input'">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {{ block.config.label }}
        </label>
        <div class="flex gap-2">
          <input 
            v-model="inputValue"
            :type="block.config.inputType || 'text'"
            :placeholder="block.config.placeholder"
            :disabled="hasResponded"
            class="flex-1 px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50"
            @keyup.enter="handleInputSubmit"
          />
          <button 
            @click="handleInputSubmit"
            :disabled="hasResponded || !inputValue.trim()"
            class="px-3 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            <Send class="w-4 h-4" />
          </button>
        </div>
        <div v-if="block.responses?.length" class="mt-3 space-y-1">
          <p 
            v-for="(resp, i) in block.responses" 
            :key="i"
            class="text-sm text-gray-600 dark:text-gray-400"
          >
            <span class="font-medium">{{ resp.responderName || 'Peer' }}:</span> {{ resp.value }}
          </p>
        </div>
      </template>
      
      <!-- Poll Widget -->
      <template v-else-if="block.config.type === 'poll'">
        <p class="font-medium text-gray-900 dark:text-white mb-3">
          {{ block.config.question }}
        </p>
        <div class="space-y-2">
          <button 
            v-for="(option, i) in block.config.options"
            :key="i"
            @click="togglePollOption(i)"
            :disabled="hasResponded"
            class="w-full text-left px-3 py-2 rounded-lg border transition-colors relative overflow-hidden disabled:cursor-default"
            :class="selectedOptions.includes(i) 
              ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30' 
              : 'border-gray-300 dark:border-gray-600 hover:border-gray-400'"
          >
            <!-- Progress bar for results -->
            <div 
              v-if="hasResponded && pollResults[i]"
              class="absolute inset-0 bg-blue-100 dark:bg-blue-900/50 transition-all"
              :style="{ width: `${pollResults[i].percentage}%` }"
            />
            <span class="relative z-10 flex items-center justify-between">
              <span class="text-sm text-gray-900 dark:text-white">{{ option }}</span>
              <span v-if="hasResponded" class="text-xs text-gray-500">
                {{ pollResults[i]?.count || 0 }} ({{ Math.round(pollResults[i]?.percentage || 0) }}%)
              </span>
            </span>
          </button>
        </div>
        <button 
          v-if="!hasResponded && selectedOptions.length > 0"
          @click="handlePollVote"
          class="mt-3 w-full px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg font-medium transition-colors"
        >
          Vote
        </button>
      </template>
      
      <!-- Progress Widget -->
      <template v-else-if="block.config.type === 'progress'">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
            {{ block.config.label }}
          </span>
          <span class="text-sm text-gray-500">
            {{ block.config.value }} / {{ block.config.max }}
          </span>
        </div>
        <div class="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div 
            class="h-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-500"
            :style="{ width: `${(block.config.value / block.config.max) * 100}%` }"
          />
        </div>
      </template>
      
      <!-- Confirm Widget -->
      <template v-else-if="block.config.type === 'confirm'">
        <p class="text-sm text-gray-700 dark:text-gray-300 mb-3">
          {{ block.config.message }}
        </p>
        <div v-if="!hasResponded" class="flex gap-2">
          <button 
            @click="handleConfirm(true)"
            class="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-green-500 hover:bg-green-600 text-white rounded-lg font-medium transition-colors"
          >
            <Check class="w-4 h-4" />
            {{ block.config.confirmLabel || 'Confirm' }}
          </button>
          <button 
            @click="handleConfirm(false)"
            class="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg font-medium transition-colors"
          >
            <X class="w-4 h-4" />
            {{ block.config.cancelLabel || 'Cancel' }}
          </button>
        </div>
        <p v-else class="text-sm text-center text-gray-500">
          Response recorded
        </p>
      </template>
    </div>
    
    <!-- Timestamp -->
    <div class="mt-1 text-xs text-gray-500" :class="block.isLocal ? 'text-right' : 'text-left'">
      {{ formattedTime }}
    </div>
  </div>
</template>
