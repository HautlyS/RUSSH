<script setup lang="ts">
import { ref, computed } from 'vue';
import { Folder, File, FileText, FileCode, FileImage, FileArchive, ChevronUp, ChevronDown } from 'lucide-vue-next';
import type { FileEntry } from '@/types/files';

const props = defineProps<{
  entries: FileEntry[];
  selectedPaths: string[];
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', entry: FileEntry, multi: boolean): void;
  (e: 'open', entry: FileEntry): void;
  (e: 'contextmenu', event: MouseEvent, entry: FileEntry): void;
  (e: 'drop', files: FileEntry[], targetPath: string): void;
}>();

type SortKey = 'name' | 'size' | 'modified';
type SortDir = 'asc' | 'desc';

const sortKey = ref<SortKey>('name');
const sortDir = ref<SortDir>('asc');

const sortedEntries = computed(() => {
  const entries = [...props.entries];
  
  entries.sort((a, b) => {
    // Directories first
    if (a.isDirectory !== b.isDirectory) {
      return a.isDirectory ? -1 : 1;
    }
    
    let cmp = 0;
    switch (sortKey.value) {
      case 'name':
        cmp = a.name.localeCompare(b.name);
        break;
      case 'size':
        cmp = (a.size || 0) - (b.size || 0);
        break;
      case 'modified':
        cmp = (a.modified || 0) - (b.modified || 0);
        break;
    }
    
    return sortDir.value === 'asc' ? cmp : -cmp;
  });
  
  return entries;
});

function toggleSort(key: SortKey) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc';
  } else {
    sortKey.value = key;
    sortDir.value = 'asc';
  }
}

function getFileIcon(entry: FileEntry) {
  if (entry.isDirectory) return Folder;
  
  const ext = entry.name.split('.').pop()?.toLowerCase();
  switch (ext) {
    case 'txt':
    case 'md':
    case 'log':
      return FileText;
    case 'js':
    case 'ts':
    case 'py':
    case 'rs':
    case 'go':
    case 'java':
    case 'c':
    case 'cpp':
    case 'h':
    case 'vue':
    case 'jsx':
    case 'tsx':
      return FileCode;
    case 'jpg':
    case 'jpeg':
    case 'png':
    case 'gif':
    case 'svg':
    case 'webp':
      return FileImage;
    case 'zip':
    case 'tar':
    case 'gz':
    case 'rar':
    case '7z':
      return FileArchive;
    default:
      return File;
  }
}

function formatSize(bytes?: number): string {
  if (bytes === undefined) return '-';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function formatDate(timestamp?: number): string {
  if (!timestamp) return '-';
  return new Date(timestamp * 1000).toLocaleString();
}

function handleClick(e: MouseEvent, entry: FileEntry) {
  emit('select', entry, e.ctrlKey || e.metaKey);
}

function handleDblClick(entry: FileEntry) {
  emit('open', entry);
}

function handleDragStart(e: DragEvent, entry: FileEntry) {
  e.dataTransfer?.setData('application/json', JSON.stringify(entry));
}

function handleDrop(e: DragEvent, targetEntry: FileEntry) {
  if (!targetEntry.isDirectory) return;
  
  const data = e.dataTransfer?.getData('application/json');
  if (data) {
    const entry = JSON.parse(data) as FileEntry;
    emit('drop', [entry], targetEntry.path);
  }
}
</script>

<template>
  <div class="file-list flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center px-3 py-2 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 text-xs font-medium text-gray-500 dark:text-gray-400">
      <button 
        class="flex-1 flex items-center gap-1 text-left hover:text-gray-700 dark:hover:text-gray-200"
        @click="toggleSort('name')"
      >
        Name
        <ChevronUp v-if="sortKey === 'name' && sortDir === 'asc'" class="w-3 h-3" />
        <ChevronDown v-else-if="sortKey === 'name'" class="w-3 h-3" />
      </button>
      <button 
        class="w-24 text-right flex items-center justify-end gap-1 hover:text-gray-700 dark:hover:text-gray-200"
        @click="toggleSort('size')"
      >
        Size
        <ChevronUp v-if="sortKey === 'size' && sortDir === 'asc'" class="w-3 h-3" />
        <ChevronDown v-else-if="sortKey === 'size'" class="w-3 h-3" />
      </button>
      <button 
        class="w-40 text-right flex items-center justify-end gap-1 hover:text-gray-700 dark:hover:text-gray-200"
        @click="toggleSort('modified')"
      >
        Modified
        <ChevronUp v-if="sortKey === 'modified' && sortDir === 'asc'" class="w-3 h-3" />
        <ChevronDown v-else-if="sortKey === 'modified'" class="w-3 h-3" />
      </button>
    </div>
    
    <!-- Loading -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <div class="animate-spin w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full" />
    </div>
    
    <!-- Empty State -->
    <div v-else-if="entries.length === 0" class="flex-1 flex items-center justify-center text-gray-500 dark:text-gray-400">
      <p>Empty directory</p>
    </div>
    
    <!-- File List -->
    <div v-else class="flex-1 overflow-auto">
      <div
        v-for="entry in sortedEntries"
        :key="entry.path"
        class="flex items-center px-3 py-2 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 border-b border-gray-100 dark:border-gray-800"
        :class="{ 'bg-blue-50 dark:bg-blue-900/20': selectedPaths.includes(entry.path) }"
        draggable="true"
        @click="handleClick($event, entry)"
        @dblclick="handleDblClick(entry)"
        @contextmenu.prevent="emit('contextmenu', $event, entry)"
        @dragstart="handleDragStart($event, entry)"
        @dragover.prevent
        @drop.prevent="handleDrop($event, entry)"
      >
        <div class="flex-1 flex items-center gap-2 min-w-0">
          <component 
            :is="getFileIcon(entry)" 
            class="w-4 h-4 flex-shrink-0"
            :class="entry.isDirectory ? 'text-yellow-500' : 'text-gray-400'"
          />
          <span class="truncate text-sm text-gray-700 dark:text-gray-300">{{ entry.name }}</span>
        </div>
        <span class="w-24 text-right text-xs text-gray-500 dark:text-gray-400">
          {{ entry.isDirectory ? '-' : formatSize(entry.size) }}
        </span>
        <span class="w-40 text-right text-xs text-gray-500 dark:text-gray-400">
          {{ formatDate(entry.modified) }}
        </span>
      </div>
    </div>
  </div>
</template>
