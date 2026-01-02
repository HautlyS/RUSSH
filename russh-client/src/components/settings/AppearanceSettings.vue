<script setup lang="ts">
import { computed } from 'vue';
import { Sun, Moon, Monitor } from 'lucide-vue-next';
import { useSettingsStore } from '@/stores/settings';
import { useTheme } from '@/composables/useTheme';

const settingsStore = useSettingsStore();
const { theme, setTheme } = useTheme();

const accentColors = [
  { name: 'Blue', value: 'blue', class: 'bg-blue-500' },
  { name: 'Purple', value: 'purple', class: 'bg-purple-500' },
  { name: 'Green', value: 'green', class: 'bg-green-500' },
  { name: 'Orange', value: 'orange', class: 'bg-orange-500' },
  { name: 'Pink', value: 'pink', class: 'bg-pink-500' },
  { name: 'Teal', value: 'teal', class: 'bg-teal-500' },
];

const appearance = computed(() => settingsStore.settings.appearance || {});

function updateAppearance(key: string, value: any) {
  settingsStore.updateSettings({ appearance: { ...appearance.value, [key]: value } });
}
</script>

<template>
  <div class="appearance-settings space-y-6">
    <div>
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Appearance</h2>
    </div>
    
    <!-- Theme -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Theme</h3>
      
      <div class="grid grid-cols-3 gap-3">
        <button
          @click="setTheme('light')"
          class="flex flex-col items-center gap-2 p-4 rounded-lg border-2 transition-colors"
          :class="theme === 'light' ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'"
        >
          <Sun class="w-6 h-6" />
          <span class="text-sm font-medium">Light</span>
        </button>
        
        <button
          @click="setTheme('dark')"
          class="flex flex-col items-center gap-2 p-4 rounded-lg border-2 transition-colors"
          :class="theme === 'dark' ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'"
        >
          <Moon class="w-6 h-6" />
          <span class="text-sm font-medium">Dark</span>
        </button>
        
        <button
          @click="setTheme('system')"
          class="flex flex-col items-center gap-2 p-4 rounded-lg border-2 transition-colors"
          :class="theme === 'system' ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'"
        >
          <Monitor class="w-6 h-6" />
          <span class="text-sm font-medium">System</span>
        </button>
      </div>
    </section>
    
    <!-- Accent Color -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Accent Color</h3>
      
      <div class="flex gap-3">
        <button
          v-for="color in accentColors"
          :key="color.value"
          @click="updateAppearance('accentColor', color.value)"
          class="w-8 h-8 rounded-full ring-2 ring-offset-2 ring-offset-white dark:ring-offset-gray-900 transition-all"
          :class="[color.class, appearance.accentColor === color.value ? 'ring-gray-400' : 'ring-transparent hover:ring-gray-200']"
          :title="color.name"
        />
      </div>
    </section>
    
    <!-- Sidebar -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Layout</h3>
      
      <div class="flex items-center justify-between">
        <div>
          <label class="text-sm font-medium text-gray-900 dark:text-white">Compact sidebar</label>
          <p class="text-xs text-gray-500 dark:text-gray-400">Use a narrower sidebar</p>
        </div>
        <button
          @click="updateAppearance('compactSidebar', !appearance.compactSidebar)"
          class="relative w-11 h-6 rounded-full transition-colors"
          :class="appearance.compactSidebar ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
        >
          <span class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform" :class="{ 'translate-x-5': appearance.compactSidebar }" />
        </button>
      </div>
      
      <div class="flex items-center justify-between">
        <div>
          <label class="text-sm font-medium text-gray-900 dark:text-white">Show status bar</label>
          <p class="text-xs text-gray-500 dark:text-gray-400">Display status bar at bottom</p>
        </div>
        <button
          @click="updateAppearance('showStatusBar', !appearance.showStatusBar)"
          class="relative w-11 h-6 rounded-full transition-colors"
          :class="appearance.showStatusBar !== false ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
        >
          <span class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform" :class="{ 'translate-x-5': appearance.showStatusBar !== false }" />
        </button>
      </div>
    </section>
  </div>
</template>
