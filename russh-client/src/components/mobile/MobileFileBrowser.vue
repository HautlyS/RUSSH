<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useFileTransfer } from '@/composables/useFileTransfer';
import { usePlatform } from '@/composables/usePlatform';
import { 
  Folder, File, ChevronLeft, RefreshCw, Upload, Download,
  MoreVertical, Trash2, Edit, Copy, FolderPlus
} from 'lucide-vue-next';
import type { FileEntry } from '@/types/files';

const props = defineProps<{
  sessionId: string;
}>();

const { hapticFeedback } = usePlatform();
const { 
  files, 
  currentPath, 
  isLoading, 
  listFiles, 
  uploadFile, 
  downloadFile,
  deleteFile,
  createDirectory 
} = useFileTransfer(props.sessionId);

const isRefreshing = ref(false);
const selectedFile = ref<FileEntry | null>(null);
const showActions = ref(false);

// Format file size
function formatSize(bytes: number): string {
  if (bytes === 0) return '—';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

// Format date
function formatDate(date: string): string {
  return new Date(date).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

// Navigate to directory
async function navigateTo(path: string) {
  hapticFeedback('light');
  await listFiles(path);
}

// Go back
async function goBack() {
  hapticFeedback('light');
  const parent = currentPath.value.split('/').slice(0, -1).join('/') || '/';
  await listFiles(parent);
}

// Pull to refresh
async function refresh() {
  isRefreshing.value = true;
  hapticFeedback('light');
  await listFiles(currentPath.value);
  isRefreshing.value = false;
}

// Handle file tap
function handleFileTap(file: FileEntry) {
  hapticFeedback('light');
  if (file.isDirectory) {
    navigateTo(file.path);
  } else {
    selectedFile.value = file;
    showActions.value = true;
  }
}

// Long press for actions
function handleLongPress(file: FileEntry) {
  hapticFeedback('medium');
  selectedFile.value = file;
  showActions.value = true;
}

// File actions
async function handleDownload() {
  if (!selectedFile.value) return;
  hapticFeedback('light');
  showActions.value = false;
  await downloadFile(selectedFile.value.path);
}

async function handleDelete() {
  if (!selectedFile.value) return;
  hapticFeedback('warning');
  showActions.value = false;
  await deleteFile(selectedFile.value.path);
  await refresh();
}

async function handleUpload() {
  hapticFeedback('light');
  // Trigger file picker
  const input = document.createElement('input');
  input.type = 'file';
  input.multiple = true;
  input.onchange = async (e) => {
    const files = (e.target as HTMLInputElement).files;
    if (files) {
      for (const file of files) {
        await uploadFile(file, currentPath.value);
      }
      await refresh();
    }
  };
  input.click();
}

async function handleCreateFolder() {
  hapticFeedback('light');
  const name = prompt('Folder name:');
  if (name) {
    await createDirectory(`${currentPath.value}/${name}`);
    await refresh();
  }
}

// Sorted files (directories first)
const sortedFiles = computed(() => {
  return [...files.value].sort((a, b) => {
    if (a.isDirectory !== b.isDirectory) {
      return a.isDirectory ? -1 : 1;
    }
    return a.name.localeCompare(b.name);
  });
});

onMounted(() => {
  listFiles('/');
});
</script>

<template>
  <div class="mobile-file-browser flex flex-col h-full bg-white dark:bg-gray-900">
    <!-- Header -->
    <div class="flex items-center gap-2 px-4 py-3 border-b border-gray-200 dark:border-gray-700">
      <button
        @click="goBack"
        :disabled="currentPath === '/'"
        class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-50"
      >
        <ChevronLeft class="w-5 h-5" />
      </button>
      
      <div class="flex-1 truncate text-sm font-medium">
        {{ currentPath || '/' }}
      </div>
      
      <button
        @click="refresh"
        :disabled="isRefreshing"
        class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800"
      >
        <RefreshCw class="w-5 h-5" :class="{ 'animate-spin': isRefreshing }" />
      </button>
    </div>
    
    <!-- File List -->
    <div class="flex-1 overflow-auto">
      <div v-if="isLoading" class="flex items-center justify-center h-32">
        <RefreshCw class="w-6 h-6 animate-spin text-gray-400" />
      </div>
      
      <div v-else-if="sortedFiles.length === 0" class="flex items-center justify-center h-32 text-gray-500">
        Empty folder
      </div>
      
      <div v-else class="divide-y divide-gray-100 dark:divide-gray-800">
        <button
          v-for="file in sortedFiles"
          :key="file.path"
          @click="handleFileTap(file)"
          @contextmenu.prevent="handleLongPress(file)"
          class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-800 text-left"
        >
          <div class="flex-shrink-0">
            <Folder v-if="file.isDirectory" class="w-6 h-6 text-blue-500" />
            <File v-else class="w-6 h-6 text-gray-400" />
          </div>
          
          <div class="flex-1 min-w-0">
            <div class="font-medium truncate">{{ file.name }}</div>
            <div class="text-xs text-gray-500 flex items-center gap-2">
              <span>{{ formatSize(file.size) }}</span>
              <span>•</span>
              <span>{{ formatDate(file.modified) }}</span>
            </div>
          </div>
          
          <button
            @click.stop="handleLongPress(file)"
            class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700"
          >
            <MoreVertical class="w-4 h-4 text-gray-400" />
          </button>
        </button>
      </div>
    </div>
    
    <!-- Bottom Actions -->
    <div class="flex items-center gap-2 px-4 py-3 border-t border-gray-200 dark:border-gray-700 safe-area-bottom">
      <button
        @click="handleUpload"
        class="flex-1 flex items-center justify-center gap-2 py-2 bg-blue-600 text-white rounded-lg"
      >
        <Upload class="w-4 h-4" />
        Upload
      </button>
      <button
        @click="handleCreateFolder"
        class="flex-1 flex items-center justify-center gap-2 py-2 bg-gray-200 dark:bg-gray-700 rounded-lg"
      >
        <FolderPlus class="w-4 h-4" />
        New Folder
      </button>
    </div>
    
    <!-- Action Sheet -->
    <Teleport to="body">
      <div
        v-if="showActions"
        class="fixed inset-0 z-50 flex items-end justify-center bg-black/50"
        @click="showActions = false"
      >
        <div
          class="w-full max-w-lg bg-white dark:bg-gray-800 rounded-t-2xl safe-area-bottom"
          @click.stop
        >
          <div class="p-4 border-b border-gray-200 dark:border-gray-700">
            <div class="font-medium truncate">{{ selectedFile?.name }}</div>
            <div class="text-sm text-gray-500">{{ formatSize(selectedFile?.size || 0) }}</div>
          </div>
          
          <div class="p-2">
            <button
              v-if="!selectedFile?.isDirectory"
              @click="handleDownload"
              class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
            >
              <Download class="w-5 h-5 text-blue-500" />
              Download
            </button>
            
            <button
              @click="handleDelete"
              class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg text-red-500"
            >
              <Trash2 class="w-5 h-5" />
              Delete
            </button>
          </div>
          
          <button
            @click="showActions = false"
            class="w-full py-4 text-center font-medium text-blue-600 border-t border-gray-200 dark:border-gray-700"
          >
            Cancel
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.safe-area-bottom {
  padding-bottom: env(safe-area-inset-bottom, 0);
}
</style>
