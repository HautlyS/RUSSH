<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { X } from 'lucide-vue-next';
import { useSettingsStore } from '@/stores/settings';
import { terminalThemes } from '@/utils/terminalThemes';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'apply'): void;
}>();

const settingsStore = useSettingsStore();

const localSettings = ref({
  fontFamily: settingsStore.settings.terminal.fontFamily,
  fontSize: settingsStore.settings.terminal.fontSize,
  theme: settingsStore.settings.terminal.theme,
  cursorStyle: settingsStore.settings.terminal.cursorStyle,
  cursorBlink: settingsStore.settings.terminal.cursorBlink,
  scrollback: settingsStore.settings.terminal.scrollback,
});

const fontFamilies = [
  'JetBrains Mono',
  'Fira Code',
  'Source Code Pro',
  'Cascadia Code',
  'Consolas',
  'Monaco',
  'monospace'
];

const cursorStyles = ['block', 'underline', 'bar'];

function applySettings() {
  settingsStore.updateSettings({
    terminal: { ...localSettings.value }
  });
  emit('apply');
}

function resetToDefaults() {
  localSettings.value = {
    fontFamily: 'JetBrains Mono',
    fontSize: 14,
    theme: 'dark',
    cursorStyle: 'block',
    cursorBlink: true,
    scrollback: 10000,
  };
}
</script>

<template>
  <Teleport to="body">
    <div 
      v-if="visible"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="emit('close')"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4">
        <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Terminal Settings</h2>
          <button
            @click="emit('close')"
            class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          >
            <X class="w-5 h-5 text-gray-500" />
          </button>
        </div>
        
        <div class="p-4 space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Font Family</label>
            <select
              v-model="localSettings.fontFamily"
              class="w-full px-3 py-2 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg"
            >
              <option v-for="font in fontFamilies" :key="font" :value="font">{{ font }}</option>
            </select>
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Font Size: {{ localSettings.fontSize }}px
            </label>
            <input
              v-model.number="localSettings.fontSize"
              type="range"
              min="10"
              max="24"
              class="w-full"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Theme</label>
            <select
              v-model="localSettings.theme"
              class="w-full px-3 py-2 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg"
            >
              <option v-for="(_, name) in terminalThemes" :key="name" :value="name">
                {{ name.charAt(0).toUpperCase() + name.slice(1) }}
              </option>
            </select>
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Cursor Style</label>
            <select
              v-model="localSettings.cursorStyle"
              class="w-full px-3 py-2 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg"
            >
              <option v-for="style in cursorStyles" :key="style" :value="style">
                {{ style.charAt(0).toUpperCase() + style.slice(1) }}
              </option>
            </select>
          </div>
          
          <div class="flex items-center justify-between">
            <label class="text-sm font-medium text-gray-700 dark:text-gray-300">Cursor Blink</label>
            <button
              @click="localSettings.cursorBlink = !localSettings.cursorBlink"
              class="relative w-11 h-6 rounded-full transition-colors"
              :class="localSettings.cursorBlink ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
            >
              <span 
                class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform"
                :class="{ 'translate-x-5': localSettings.cursorBlink }"
              />
            </button>
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Scrollback Lines: {{ localSettings.scrollback.toLocaleString() }}
            </label>
            <input
              v-model.number="localSettings.scrollback"
              type="range"
              min="1000"
              max="100000"
              step="1000"
              class="w-full"
            />
          </div>
        </div>
        
        <div class="flex items-center justify-between px-4 py-3 border-t border-gray-200 dark:border-gray-700">
          <button
            @click="resetToDefaults"
            class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          >
            Reset to Defaults
          </button>
          <div class="flex gap-2">
            <button
              @click="emit('close')"
              class="px-4 py-2 text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              @click="applySettings"
              class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
            >
              Apply
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
