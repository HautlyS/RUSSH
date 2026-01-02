<script setup lang="ts">
import { ref } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { useVisualEffects } from '@/composables/useVisualEffects';
import Stepper from '@/components/extra/Stepper.vue';
import { Sun, Moon, Monitor, Terminal, Bell, Sparkles } from 'lucide-vue-next';

const emit = defineEmits<{
  (e: 'complete'): void;
  (e: 'skip'): void;
}>();

const settingsStore = useSettingsStore();
const { isStepperEnabled, updateVisualEffects, visualEffects } = useVisualEffects();

const selectedTheme = ref(settingsStore.settings.appearance.theme);
const selectedAccent = ref(settingsStore.settings.appearance.accentColor);
const notificationsEnabled = ref(settingsStore.settings.notifications.enabled);
const visualEffectsEnabled = ref(visualEffects.value.globalEnabled);

const accentColors = [
  { name: 'Blue', value: '#3b82f6' },
  { name: 'Green', value: '#27FF64' },
  { name: 'Purple', value: '#a855f7' },
  { name: 'Pink', value: '#ec4899' },
  { name: 'Orange', value: '#f97316' },
  { name: 'Cyan', value: '#06b6d4' },
];

function handleStepChange(step: number) {
  // Save settings as user progresses
  if (step > 1) {
    settingsStore.updateSettings({
      appearance: {
        ...settingsStore.settings.appearance,
        theme: selectedTheme.value,
        accentColor: selectedAccent.value,
      },
    });
  }
  if (step > 2) {
    settingsStore.updateSettings({
      notifications: {
        ...settingsStore.settings.notifications,
        enabled: notificationsEnabled.value,
      },
    });
  }
}

function handleComplete() {
  // Save all settings
  settingsStore.updateSettings({
    appearance: {
      ...settingsStore.settings.appearance,
      theme: selectedTheme.value,
      accentColor: selectedAccent.value,
    },
    notifications: {
      ...settingsStore.settings.notifications,
      enabled: notificationsEnabled.value,
    },
  });
  
  updateVisualEffects({
    globalEnabled: visualEffectsEnabled.value,
  });
  
  emit('complete');
}
</script>

<template>
  <div class="setup-wizard">
    <Stepper
      v-if="isStepperEnabled"
      :initial-step="1"
      :on-step-change="handleStepChange"
      :on-final-step-completed="handleComplete"
      back-button-text="Back"
      next-button-text="Next"
      step-circle-container-class-name="bg-gray-900"
    >
      <!-- Step 1: Welcome -->
      <div class="text-center py-4">
        <div class="w-16 h-16 mx-auto mb-4 bg-gradient-to-br from-blue-500 to-purple-600 rounded-2xl flex items-center justify-center">
          <Terminal class="w-8 h-8 text-white" />
        </div>
        <h2 class="text-xl font-bold text-white mb-2">Welcome to RUSSH</h2>
        <p class="text-gray-400">Let's set up your SSH client in a few quick steps.</p>
      </div>

      <!-- Step 2: Theme -->
      <div class="py-4">
        <h2 class="text-lg font-semibold text-white mb-4 text-center">Choose Your Theme</h2>
        <div class="grid grid-cols-3 gap-3">
          <button
            @click="selectedTheme = 'light'"
            class="p-4 rounded-xl border-2 transition-all flex flex-col items-center gap-2"
            :class="selectedTheme === 'light' ? 'border-green-500 bg-green-500/10' : 'border-gray-700 hover:border-gray-600'"
          >
            <Sun class="w-6 h-6 text-yellow-400" />
            <span class="text-sm text-gray-300">Light</span>
          </button>
          <button
            @click="selectedTheme = 'dark'"
            class="p-4 rounded-xl border-2 transition-all flex flex-col items-center gap-2"
            :class="selectedTheme === 'dark' ? 'border-green-500 bg-green-500/10' : 'border-gray-700 hover:border-gray-600'"
          >
            <Moon class="w-6 h-6 text-blue-400" />
            <span class="text-sm text-gray-300">Dark</span>
          </button>
          <button
            @click="selectedTheme = 'system'"
            class="p-4 rounded-xl border-2 transition-all flex flex-col items-center gap-2"
            :class="selectedTheme === 'system' ? 'border-green-500 bg-green-500/10' : 'border-gray-700 hover:border-gray-600'"
          >
            <Monitor class="w-6 h-6 text-gray-400" />
            <span class="text-sm text-gray-300">System</span>
          </button>
        </div>
      </div>

      <!-- Step 3: Accent Color -->
      <div class="py-4">
        <h2 class="text-lg font-semibold text-white mb-4 text-center">Pick an Accent Color</h2>
        <div class="grid grid-cols-3 gap-3">
          <button
            v-for="color in accentColors"
            :key="color.value"
            @click="selectedAccent = color.value"
            class="p-4 rounded-xl border-2 transition-all flex flex-col items-center gap-2"
            :class="selectedAccent === color.value ? 'border-green-500 bg-green-500/10' : 'border-gray-700 hover:border-gray-600'"
          >
            <div class="w-8 h-8 rounded-full" :style="{ backgroundColor: color.value }" />
            <span class="text-sm text-gray-300">{{ color.name }}</span>
          </button>
        </div>
      </div>

      <!-- Step 4: Notifications & Effects -->
      <div class="py-4 space-y-4">
        <h2 class="text-lg font-semibold text-white mb-4 text-center">Final Touches</h2>
        
        <div class="flex items-center justify-between p-4 bg-gray-800 rounded-xl">
          <div class="flex items-center gap-3">
            <Bell class="w-5 h-5 text-gray-400" />
            <div>
              <p class="text-white font-medium">Notifications</p>
              <p class="text-sm text-gray-400">Get alerts for connections</p>
            </div>
          </div>
          <button
            @click="notificationsEnabled = !notificationsEnabled"
            class="relative w-11 h-6 rounded-full transition-colors"
            :class="notificationsEnabled ? 'bg-green-500' : 'bg-gray-600'"
          >
            <span 
              class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform"
              :class="{ 'translate-x-5': notificationsEnabled }"
            />
          </button>
        </div>

        <div class="flex items-center justify-between p-4 bg-gray-800 rounded-xl">
          <div class="flex items-center gap-3">
            <Sparkles class="w-5 h-5 text-gray-400" />
            <div>
              <p class="text-white font-medium">Visual Effects</p>
              <p class="text-sm text-gray-400">Enable animations & effects</p>
            </div>
          </div>
          <button
            @click="visualEffectsEnabled = !visualEffectsEnabled"
            class="relative w-11 h-6 rounded-full transition-colors"
            :class="visualEffectsEnabled ? 'bg-green-500' : 'bg-gray-600'"
          >
            <span 
              class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform"
              :class="{ 'translate-x-5': visualEffectsEnabled }"
            />
          </button>
        </div>
      </div>
    </Stepper>

    <!-- Fallback without Stepper -->
    <div v-else class="p-6 text-center">
      <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">Setup Complete</h2>
      <p class="text-gray-500 mb-4">You can customize settings anytime from the Settings panel.</p>
      <button @click="emit('complete')" class="btn-primary">Get Started</button>
    </div>
  </div>
</template>
