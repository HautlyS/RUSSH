<script setup lang="ts">
import { computed } from 'vue';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();
const settings = computed(() => settingsStore.settings);

function updateSetting(key: string, value: any) {
  settingsStore.updateSettings({ [key]: value });
}
</script>

<template>
  <div class="general-settings space-y-6">
    <div>
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">General Settings</h2>
    </div>
    
    <!-- Startup -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Startup</h3>
      
      <div class="flex items-center justify-between">
        <div>
          <label class="text-sm font-medium text-gray-900 dark:text-white">Start minimized</label>
          <p class="text-xs text-gray-500 dark:text-gray-400">Start the app minimized to system tray</p>
        </div>
        <button
          @click="updateSetting('startMinimized', !settings.startMinimized)"
          class="relative w-11 h-6 rounded-full transition-colors"
          :class="settings.startMinimized ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
        >
          <span class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform" :class="{ 'translate-x-5': settings.startMinimized }" />
        </button>
      </div>
      
      <div class="flex items-center justify-between">
        <div>
          <label class="text-sm font-medium text-gray-900 dark:text-white">Auto-connect</label>
          <p class="text-xs text-gray-500 dark:text-gray-400">Automatically connect to last used servers</p>
        </div>
        <button
          @click="updateSetting('autoConnect', !settings.autoConnect)"
          class="relative w-11 h-6 rounded-full transition-colors"
          :class="settings.autoConnect ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
        >
          <span class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform" :class="{ 'translate-x-5': settings.autoConnect }" />
        </button>
      </div>
    </section>
    
    <!-- Updates -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Updates</h3>
      
      <div class="flex items-center justify-between">
        <div>
          <label class="text-sm font-medium text-gray-900 dark:text-white">Check for updates</label>
          <p class="text-xs text-gray-500 dark:text-gray-400">Automatically check for app updates</p>
        </div>
        <button
          @click="updateSetting('checkUpdates', !settings.checkUpdates)"
          class="relative w-11 h-6 rounded-full transition-colors"
          :class="settings.checkUpdates ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
        >
          <span class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform" :class="{ 'translate-x-5': settings.checkUpdates }" />
        </button>
      </div>
    </section>
    
    <!-- Notifications -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Notifications</h3>
      
      <div class="flex items-center justify-between">
        <div>
          <label class="text-sm font-medium text-gray-900 dark:text-white">Enable notifications</label>
          <p class="text-xs text-gray-500 dark:text-gray-400">Show system notifications for events</p>
        </div>
        <button
          @click="updateSetting('notifications', { ...settings.notifications, enabled: !settings.notifications?.enabled })"
          class="relative w-11 h-6 rounded-full transition-colors"
          :class="settings.notifications?.enabled ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
        >
          <span class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform" :class="{ 'translate-x-5': settings.notifications?.enabled }" />
        </button>
      </div>
    </section>
  </div>
</template>
