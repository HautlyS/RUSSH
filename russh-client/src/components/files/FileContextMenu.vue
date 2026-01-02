<script setup lang="ts">
import { computed } from 'vue';
import { 
  Download, Upload, Trash2, Edit, Copy, Scissors, Clipboard, 
  FolderPlus, FilePlus, RefreshCw, Eye, Lock 
} from 'lucide-vue-next';
import type { FileEntry } from '@/types/files';

const props = defineProps<{
  visible: boolean;
  x: number;
  y: number;
  entry?: FileEntry;
  hasClipboard?: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'download'): void;
  (e: 'upload'): void;
  (e: 'delete'): void;
  (e: 'rename'): void;
  (e: 'copy'): void;
  (e: 'cut'): void;
  (e: 'paste'): void;
  (e: 'newFolder'): void;
  (e: 'newFile'): void;
  (e: 'refresh'): void;
  (e: 'properties'): void;
  (e: 'chmod'): void;
}>();

const menuItems = computed(() => {
  const items = [];
  
  if (props.entry) {
    if (!props.entry.isDirectory) {
      items.push({ icon: Download, label: 'Download', action: 'download' });
    }
    items.push({ icon: Edit, label: 'Rename', action: 'rename', shortcut: 'F2' });
    items.push({ icon: Copy, label: 'Copy', action: 'copy', shortcut: 'Ctrl+C' });
    items.push({ icon: Scissors, label: 'Cut', action: 'cut', shortcut: 'Ctrl+X' });
    items.push({ divider: true });
    items.push({ icon: Lock, label: 'Permissions', action: 'chmod' });
    items.push({ icon: Eye, label: 'Properties', action: 'properties' });
    items.push({ divider: true });
    items.push({ icon: Trash2, label: 'Delete', action: 'delete', shortcut: 'Del', danger: true });
  } else {
    items.push({ icon: Upload, label: 'Upload files', action: 'upload' });
    items.push({ icon: FolderPlus, label: 'New folder', action: 'newFolder' });
    items.push({ icon: FilePlus, label: 'New file', action: 'newFile' });
    items.push({ divider: true });
    if (props.hasClipboard) {
      items.push({ icon: Clipboard, label: 'Paste', action: 'paste', shortcut: 'Ctrl+V' });
    }
    items.push({ icon: RefreshCw, label: 'Refresh', action: 'refresh', shortcut: 'F5' });
  }
  
  return items;
});

function handleAction(action: string) {
  emit(action as any);
  emit('close');
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="fixed inset-0 z-50"
      @click="emit('close')"
      @contextmenu.prevent="emit('close')"
    >
      <div
        class="absolute bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 min-w-[180px]"
        :style="{ left: `${x}px`, top: `${y}px` }"
        @click.stop
      >
        <template v-for="(item, index) in menuItems" :key="index">
          <div v-if="item.divider" class="my-1 border-t border-gray-200 dark:border-gray-700" />
          <button
            v-else
            class="w-full flex items-center gap-3 px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
            :class="item.danger ? 'text-red-600 dark:text-red-400' : 'text-gray-700 dark:text-gray-300'"
            @click="handleAction(item.action!)"
          >
            <component :is="item.icon" class="w-4 h-4" />
            <span class="flex-1 text-left">{{ item.label }}</span>
            <span v-if="item.shortcut" class="text-xs text-gray-400">{{ item.shortcut }}</span>
          </button>
        </template>
      </div>
    </div>
  </Teleport>
</template>
