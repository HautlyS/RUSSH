<script setup lang="ts">
import { computed } from 'vue';
import { Upload, Download, X, Check, AlertCircle, Pause, Play } from 'lucide-vue-next';
import type { TransferItem } from '@/types/files';

const props = defineProps<{
  transfers: TransferItem[];
}>();

const emit = defineEmits<{
  (e: 'cancel', id: string): void;
  (e: 'pause', id: string): void;
  (e: 'resume', id: string): void;
  (e: 'retry', id: string): void;
  (e: 'clear'): void;
}>();

const activeTransfers = computed(() => 
  props.transfers.filter(t => t.status === 'pending' || t.status === 'transferring')
);

const completedTransfers = computed(() => 
  props.transfers.filter(t => t.status === 'completed' || t.status === 'failed')
);

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function formatSpeed(bytesPerSec: number): string {
  return `${formatSize(bytesPerSec)}/s`;
}

function getStatusIcon(status: TransferItem['status']) {
  switch (status) {
    case 'completed': return Check;
    case 'failed': return AlertCircle;
    default: return null;
  }
}
</script>

<template>
  <div class="file-transfer-queue bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
    <div class="flex items-center justify-between px-3 py-2 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">
        Transfers
        <span v-if="activeTransfers.length" class="ml-1 text-blue-500">({{ activeTransfers.length }} active)</span>
      </h3>
      <button
        v-if="completedTransfers.length"
        @click="emit('clear')"
        class="text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
      >
        Clear completed
      </button>
    </div>
    
    <div v-if="transfers.length === 0" class="p-4 text-center text-sm text-gray-500 dark:text-gray-400">
      No transfers
    </div>
    
    <div v-else class="max-h-64 overflow-auto">
      <div
        v-for="transfer in transfers"
        :key="transfer.id"
        class="flex items-center gap-3 px-3 py-2 border-b border-gray-100 dark:border-gray-700 last:border-0"
      >
        <div class="flex-shrink-0">
          <Upload v-if="transfer.direction === 'upload'" class="w-4 h-4 text-blue-500" />
          <Download v-else class="w-4 h-4 text-green-500" />
        </div>
        
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm truncate text-gray-700 dark:text-gray-300">{{ transfer.name }}</span>
            <component 
              v-if="getStatusIcon(transfer.status)"
              :is="getStatusIcon(transfer.status)"
              class="w-4 h-4 flex-shrink-0"
              :class="transfer.status === 'completed' ? 'text-green-500' : 'text-red-500'"
            />
          </div>
          
          <div v-if="transfer.status === 'transferring'" class="mt-1">
            <div class="h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
              <div 
                class="h-full bg-blue-500 transition-all duration-300"
                :style="{ width: `${transfer.progress}%` }"
              />
            </div>
            <div class="flex justify-between mt-0.5 text-xs text-gray-500 dark:text-gray-400">
              <span>{{ formatSize(transfer.transferred) }} / {{ formatSize(transfer.size) }}</span>
              <span v-if="transfer.speed">{{ formatSpeed(transfer.speed) }}</span>
            </div>
          </div>
          
          <div v-else-if="transfer.status === 'failed'" class="text-xs text-red-500 mt-0.5">
            {{ transfer.error || 'Transfer failed' }}
          </div>
          
          <div v-else-if="transfer.status === 'completed'" class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
            {{ formatSize(transfer.size) }}
          </div>
        </div>
        
        <div class="flex items-center gap-1">
          <button
            v-if="transfer.status === 'transferring'"
            @click="emit('pause', transfer.id)"
            class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500"
            title="Pause"
          >
            <Pause class="w-4 h-4" />
          </button>
          
          <button
            v-if="transfer.status === 'paused'"
            @click="emit('resume', transfer.id)"
            class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500"
            title="Resume"
          >
            <Play class="w-4 h-4" />
          </button>
          
          <button
            v-if="transfer.status === 'failed'"
            @click="emit('retry', transfer.id)"
            class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500"
            title="Retry"
          >
            <Play class="w-4 h-4" />
          </button>
          
          <button
            v-if="transfer.status !== 'completed'"
            @click="emit('cancel', transfer.id)"
            class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 hover:text-red-500"
            title="Cancel"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
