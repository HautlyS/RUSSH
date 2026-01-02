<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { ChevronLeft, ChevronRight, Home, RefreshCw, Upload, FolderPlus, Grid, List } from 'lucide-vue-next';
import { useFileTransfer } from '@/composables/useFileTransfer';
import FileList from './FileList.vue';
import FileTree from './FileTree.vue';
import FileContextMenu from './FileContextMenu.vue';
import FileTransferQueue from './FileTransferQueue.vue';
import type { FileEntry } from '@/types/files';

const props = defineProps<{
  sessionId: string;
  initialPath?: string;
}>();

const { listFiles, uploadFile, downloadFile, deleteFile, createDirectory, transfers, isLoading } = useFileTransfer();

const currentPath = ref(props.initialPath || '/');
const entries = ref<FileEntry[]>([]);
const selectedPaths = ref<string[]>([]);
const history = ref<string[]>(['/']);
const historyIndex = ref(0);
const viewMode = ref<'list' | 'grid'>('list');
const showTree = ref(true);

// Context menu state
const contextMenu = ref({ visible: false, x: 0, y: 0, entry: undefined as FileEntry | undefined });
const clipboard = ref<{ entries: FileEntry[]; action: 'copy' | 'cut' } | null>(null);

async function loadDirectory(path: string) {
  try {
    entries.value = await listFiles(props.sessionId, path);
    currentPath.value = path;
  } catch (err) {
    console.error('Failed to load directory:', err);
  }
}

function navigateTo(path: string) {
  if (historyIndex.value < history.value.length - 1) {
    history.value = history.value.slice(0, historyIndex.value + 1);
  }
  history.value.push(path);
  historyIndex.value = history.value.length - 1;
  loadDirectory(path);
}

function goBack() {
  if (historyIndex.value > 0) {
    historyIndex.value--;
    loadDirectory(history.value[historyIndex.value]);
  }
}

function goForward() {
  if (historyIndex.value < history.value.length - 1) {
    historyIndex.value++;
    loadDirectory(history.value[historyIndex.value]);
  }
}

function goHome() {
  navigateTo('/home');
}

function refresh() {
  loadDirectory(currentPath.value);
}

function handleSelect(entry: FileEntry, multi: boolean) {
  if (multi) {
    const idx = selectedPaths.value.indexOf(entry.path);
    if (idx >= 0) {
      selectedPaths.value.splice(idx, 1);
    } else {
      selectedPaths.value.push(entry.path);
    }
  } else {
    selectedPaths.value = [entry.path];
  }
}

function handleOpen(entry: FileEntry) {
  if (entry.isDirectory) {
    navigateTo(entry.path);
  } else {
    downloadFile(props.sessionId, entry.path, entry.name);
  }
}

function handleContextMenu(event: MouseEvent, entry?: FileEntry) {
  contextMenu.value = { visible: true, x: event.clientX, y: event.clientY, entry };
}

async function handleUpload() {
  const input = document.createElement('input');
  input.type = 'file';
  input.multiple = true;
  input.onchange = async () => {
    if (input.files) {
      for (const file of input.files) {
        await uploadFile(props.sessionId, currentPath.value, file);
      }
      refresh();
    }
  };
  input.click();
}

async function handleNewFolder() {
  const name = prompt('Folder name:');
  if (name) {
    await createDirectory(props.sessionId, `${currentPath.value}/${name}`);
    refresh();
  }
}

async function handleDelete() {
  if (contextMenu.value.entry) {
    if (confirm(`Delete ${contextMenu.value.entry.name}?`)) {
      await deleteFile(props.sessionId, contextMenu.value.entry.path);
      refresh();
    }
  }
}

onMounted(() => loadDirectory(currentPath.value));
watch(() => props.sessionId, () => loadDirectory('/'));
</script>

<template>
  <div class="file-browser flex flex-col h-full bg-white dark:bg-gray-900">
    <!-- Toolbar -->
    <div class="flex items-center gap-2 px-3 py-2 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
      <button @click="goBack" :disabled="historyIndex === 0" class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50">
        <ChevronLeft class="w-4 h-4" />
      </button>
      <button @click="goForward" :disabled="historyIndex >= history.length - 1" class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50">
        <ChevronRight class="w-4 h-4" />
      </button>
      <button @click="goHome" class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700">
        <Home class="w-4 h-4" />
      </button>
      
      <div class="flex-1 px-3 py-1.5 bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded text-sm">
        {{ currentPath }}
      </div>
      
      <button @click="refresh" class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700">
        <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': isLoading }" />
      </button>
      <button @click="handleUpload" class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700">
        <Upload class="w-4 h-4" />
      </button>
      <button @click="handleNewFolder" class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700">
        <FolderPlus class="w-4 h-4" />
      </button>
      <button @click="viewMode = viewMode === 'list' ? 'grid' : 'list'" class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700">
        <Grid v-if="viewMode === 'list'" class="w-4 h-4" />
        <List v-else class="w-4 h-4" />
      </button>
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Tree Panel -->
      <div v-if="showTree" class="w-48 border-r border-gray-200 dark:border-gray-700 overflow-auto">
        <FileTree :entries="entries" :selected-path="selectedPaths[0]" @select="handleSelect($event, false)" @navigate="navigateTo" />
      </div>
      
      <!-- File List -->
      <div class="flex-1 overflow-hidden" @contextmenu.prevent="handleContextMenu($event)">
        <FileList :entries="entries" :selected-paths="selectedPaths" :loading="isLoading" @select="handleSelect" @open="handleOpen" @contextmenu="handleContextMenu" />
      </div>
    </div>
    
    <!-- Transfer Queue -->
    <div v-if="transfers.length" class="border-t border-gray-200 dark:border-gray-700">
      <FileTransferQueue :transfers="transfers" />
    </div>
    
    <!-- Context Menu -->
    <FileContextMenu v-bind="contextMenu" :has-clipboard="!!clipboard" @close="contextMenu.visible = false" @download="handleOpen(contextMenu.entry!)" @delete="handleDelete" @upload="handleUpload" @new-folder="handleNewFolder" @refresh="refresh" />
  </div>
</template>
