<script setup lang="ts">
import { ref, computed } from 'vue';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();

const defaultShortcuts = [
  { id: 'newConnection', label: 'New Connection', default: 'Ctrl+N' },
  { id: 'quickConnect', label: 'Quick Connect', default: 'Ctrl+Shift+N' },
  { id: 'closeTab', label: 'Close Tab', default: 'Ctrl+W' },
  { id: 'nextTab', label: 'Next Tab', default: 'Ctrl+Tab' },
  { id: 'prevTab', label: 'Previous Tab', default: 'Ctrl+Shift+Tab' },
  { id: 'commandPalette', label: 'Command Palette', default: 'Ctrl+K' },
  { id: 'search', label: 'Search', default: 'Ctrl+F' },
  { id: 'settings', label: 'Settings', default: 'Ctrl+,' },
  { id: 'clearTerminal', label: 'Clear Terminal', default: 'Ctrl+L' },
  { id: 'copySelection', label: 'Copy', default: 'Ctrl+Shift+C' },
  { id: 'paste', label: 'Paste', default: 'Ctrl+Shift+V' },
];

const shortcuts = computed(() => settingsStore.settings.shortcuts || {});
const editingId = ref<string | null>(null);
const recordedKeys = ref('');

function startRecording(id: string) {
  editingId.value = id;
  recordedKeys.value = '';
  window.addEventListener('keydown', recordKey);
}

function recordKey(e: KeyboardEvent) {
  e.preventDefault();
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  if (e.key !== 'Control' && e.key !== 'Alt' && e.key !== 'Shift' && e.key !== 'Meta') {
    parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);
  }
  recordedKeys.value = parts.join('+');
}

function saveShortcut() {
  if (editingId.value && recordedKeys.value) {
    settingsStore.updateSettings({
      shortcuts: { ...shortcuts.value, [editingId.value]: recordedKeys.value }
    });
  }
  cancelRecording();
}

function cancelRecording() {
  editingId.value = null;
  recordedKeys.value = '';
  window.removeEventListener('keydown', recordKey);
}

function resetShortcut(id: string) {
  const shortcut = defaultShortcuts.find(s => s.id === id);
  if (shortcut) {
    settingsStore.updateSettings({
      shortcuts: { ...shortcuts.value, [id]: shortcut.default }
    });
  }
}

function getShortcut(id: string): string {
  return shortcuts.value[id] || defaultShortcuts.find(s => s.id === id)?.default || '';
}
</script>

<template>
  <div class="keyboard-settings space-y-6">
    <div>
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Keyboard Shortcuts</h2>
      <p class="text-sm text-gray-500 dark:text-gray-400">Customize keyboard shortcuts for common actions</p>
    </div>
    
    <div class="space-y-2">
      <div
        v-for="shortcut in defaultShortcuts"
        :key="shortcut.id"
        class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg"
      >
        <span class="text-sm font-medium text-gray-900 dark:text-white">{{ shortcut.label }}</span>
        
        <div class="flex items-center gap-2">
          <template v-if="editingId === shortcut.id">
            <kbd class="px-3 py-1.5 bg-white dark:bg-gray-900 border border-blue-500 rounded text-sm font-mono min-w-[100px] text-center">
              {{ recordedKeys || 'Press keys...' }}
            </kbd>
            <button @click="saveShortcut" class="px-2 py-1 text-xs bg-blue-600 text-white rounded">Save</button>
            <button @click="cancelRecording" class="px-2 py-1 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 rounded">Cancel</button>
          </template>
          <template v-else>
            <kbd class="px-3 py-1.5 bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded text-sm font-mono">
              {{ getShortcut(shortcut.id) }}
            </kbd>
            <button @click="startRecording(shortcut.id)" class="px-2 py-1 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 rounded">Edit</button>
            <button @click="resetShortcut(shortcut.id)" class="px-2 py-1 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 rounded">Reset</button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
