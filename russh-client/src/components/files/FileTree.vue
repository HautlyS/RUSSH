<script setup lang="ts">
import { ref, computed } from 'vue';
import { ChevronRight, ChevronDown, Folder, FolderOpen, File } from 'lucide-vue-next';
import type { FileEntry } from '@/types/files';

const props = defineProps<{
  entries: FileEntry[];
  selectedPath?: string;
}>();

const emit = defineEmits<{
  (e: 'select', entry: FileEntry): void;
  (e: 'navigate', path: string): void;
}>();

const expandedDirs = ref<Set<string>>(new Set());

const sortedEntries = computed(() => {
  return [...props.entries].sort((a, b) => {
    if (a.isDirectory !== b.isDirectory) {
      return a.isDirectory ? -1 : 1;
    }
    return a.name.localeCompare(b.name);
  });
});

function toggleDir(entry: FileEntry) {
  if (entry.isDirectory) {
    if (expandedDirs.value.has(entry.path)) {
      expandedDirs.value.delete(entry.path);
    } else {
      expandedDirs.value.add(entry.path);
      emit('navigate', entry.path);
    }
  }
}

function handleSelect(entry: FileEntry) {
  emit('select', entry);
  if (entry.isDirectory) {
    toggleDir(entry);
  }
}

function getFileIcon(entry: FileEntry) {
  if (entry.isDirectory) {
    return expandedDirs.value.has(entry.path) ? FolderOpen : Folder;
  }
  return File;
}
</script>

<template>
  <div class="file-tree text-sm">
    <div
      v-for="entry in sortedEntries"
      :key="entry.path"
      class="file-tree-item"
    >
      <div
        class="flex items-center gap-1 px-2 py-1 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
        :class="{ 'bg-blue-100 dark:bg-blue-900/30': selectedPath === entry.path }"
        @click="handleSelect(entry)"
        @dblclick="entry.isDirectory && emit('navigate', entry.path)"
      >
        <button
          v-if="entry.isDirectory"
          class="p-0.5"
          @click.stop="toggleDir(entry)"
        >
          <ChevronDown v-if="expandedDirs.has(entry.path)" class="w-3 h-3 text-gray-500" />
          <ChevronRight v-else class="w-3 h-3 text-gray-500" />
        </button>
        <span v-else class="w-4" />
        
        <component 
          :is="getFileIcon(entry)" 
          class="w-4 h-4"
          :class="entry.isDirectory ? 'text-yellow-500' : 'text-gray-400'"
        />
        <span class="truncate text-gray-700 dark:text-gray-300">{{ entry.name }}</span>
      </div>
    </div>
  </div>
</template>
