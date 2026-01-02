<script setup lang="ts">
import { computed } from 'vue';
import { 
  File, 
  FileImage, 
  FileVideo, 
  FileAudio, 
  FileText, 
  FileArchive,
  Download,
  RefreshCw,
  CheckCircle,
  XCircle,
  Loader2
} from 'lucide-vue-next';
import type { FileBlock } from '@/types/blocks';

const props = defineProps<{
  block: FileBlock;
}>();

const emit = defineEmits<{
  download: [block: FileBlock];
  retry: [block: FileBlock];
}>();

const formattedSize = computed(() => {
  const bytes = props.block.size;
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
});

const formattedTime = computed(() => {
  return new Date(props.block.timestamp).toLocaleTimeString([], { 
    hour: '2-digit', 
    minute: '2-digit' 
  });
});

const fileIcon = computed(() => {
  const mime = props.block.mimeType;
  if (mime.startsWith('image/')) return FileImage;
  if (mime.startsWith('video/')) return FileVideo;
  if (mime.startsWith('audio/')) return FileAudio;
  if (mime.startsWith('text/')) return FileText;
  if (mime.includes('zip') || mime.includes('tar') || mime.includes('rar')) return FileArchive;
  return File;
});

const statusIcon = computed(() => {
  switch (props.block.transferStatus) {
    case 'completed': return CheckCircle;
    case 'failed': return XCircle;
    case 'transferring': return Loader2;
    default: return null;
  }
});

const statusColor = computed(() => {
  switch (props.block.transferStatus) {
    case 'completed': return 'text-green-500';
    case 'failed': return 'text-red-500';
    case 'transferring': return 'text-blue-500';
    default: return 'text-gray-400';
  }
});

function handleDownload() {
  emit('download', props.block);
}

function handleRetry() {
  emit('retry', props.block);
}
</script>

<template>
  <div 
    class="file-block w-full max-w-sm"
    :class="block.isLocal ? 'ml-auto' : 'mr-auto'"
  >
    <div 
      class="rounded-lg overflow-hidden border"
      :class="block.isLocal 
        ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800' 
        : 'bg-gray-100 dark:bg-gray-800 border-gray-200 dark:border-gray-700'"
    >
      <!-- Image Preview -->
      <div 
        v-if="block.previewUrl && block.mimeType.startsWith('image/')"
        class="relative aspect-video bg-gray-200 dark:bg-gray-700"
      >
        <img 
          :src="block.previewUrl" 
          :alt="block.filename"
          class="w-full h-full object-cover"
        />
        <div 
          v-if="block.transferStatus === 'transferring'"
          class="absolute inset-0 bg-black/50 flex items-center justify-center"
        >
          <div class="text-white text-center">
            <Loader2 class="w-8 h-8 animate-spin mx-auto mb-2" />
            <span class="text-sm">{{ block.transferProgress }}%</span>
          </div>
        </div>
      </div>
      
      <!-- File Info -->
      <div class="p-3 flex items-center gap-3">
        <div 
          class="w-10 h-10 rounded-lg flex items-center justify-center"
          :class="block.isLocal ? 'bg-blue-100 dark:bg-blue-800' : 'bg-gray-200 dark:bg-gray-700'"
        >
          <component 
            :is="fileIcon" 
            class="w-5 h-5"
            :class="block.isLocal ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'"
          />
        </div>
        
        <div class="flex-1 min-w-0">
          <p class="font-medium text-sm text-gray-900 dark:text-white truncate">
            {{ block.filename }}
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            {{ formattedSize }}
          </p>
        </div>
        
        <!-- Status/Actions -->
        <div class="flex items-center gap-2">
          <component 
            v-if="statusIcon && block.transferStatus !== 'pending'"
            :is="statusIcon" 
            class="w-5 h-5"
            :class="[statusColor, block.transferStatus === 'transferring' ? 'animate-spin' : '']"
          />
          
          <button 
            v-if="!block.isLocal && block.transferStatus === 'completed'"
            @click="handleDownload"
            class="p-2 text-gray-500 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/30 rounded-lg transition-colors"
            title="Download"
          >
            <Download class="w-4 h-4" />
          </button>
          
          <button 
            v-if="block.transferStatus === 'failed'"
            @click="handleRetry"
            class="p-2 text-gray-500 hover:text-orange-500 hover:bg-orange-50 dark:hover:bg-orange-900/30 rounded-lg transition-colors"
            title="Retry"
          >
            <RefreshCw class="w-4 h-4" />
          </button>
        </div>
      </div>
      
      <!-- Progress Bar -->
      <div 
        v-if="block.transferStatus === 'transferring'"
        class="px-3 pb-3"
      >
        <div class="h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div 
            class="h-full bg-blue-500 transition-all duration-300"
            :style="{ width: `${block.transferProgress || 0}%` }"
          />
        </div>
      </div>
    </div>
    
    <!-- Timestamp -->
    <div class="mt-1 text-xs text-gray-500" :class="block.isLocal ? 'text-right' : 'text-left'">
      {{ formattedTime }}
    </div>
  </div>
</template>
