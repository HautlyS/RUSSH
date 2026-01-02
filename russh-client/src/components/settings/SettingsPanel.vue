<script setup lang="ts">
import { ref } from 'vue';
import { Settings, Terminal, Palette, Keyboard, Sparkles } from 'lucide-vue-next';
import GeneralSettings from './GeneralSettings.vue';
import TerminalSettingsPanel from './TerminalSettingsPanel.vue';
import AppearanceSettings from './AppearanceSettings.vue';
import KeyboardSettings from './KeyboardSettings.vue';
import VisualEffectsSettings from './VisualEffectsSettings.vue';

const activeSection = ref('general');

const sections = [
  { id: 'general', label: 'General', icon: Settings },
  { id: 'terminal', label: 'Terminal', icon: Terminal },
  { id: 'appearance', label: 'Appearance', icon: Palette },
  { id: 'keyboard', label: 'Keyboard', icon: Keyboard },
  { id: 'visualEffects', label: 'Visual Effects', icon: Sparkles },
];
</script>

<template>
  <div class="settings-panel flex h-full">
    <!-- Sidebar -->
    <nav class="w-48 bg-gray-50 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 p-2">
      <button
        v-for="section in sections"
        :key="section.id"
        @click="activeSection = section.id"
        class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left transition-colors"
        :class="activeSection === section.id 
          ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300' 
          : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'"
      >
        <component :is="section.icon" class="w-5 h-5" />
        <span class="text-sm font-medium">{{ section.label }}</span>
      </button>
    </nav>
    
    <!-- Content -->
    <div class="flex-1 overflow-auto p-6">
      <GeneralSettings v-if="activeSection === 'general'" />
      <TerminalSettingsPanel v-else-if="activeSection === 'terminal'" />
      <AppearanceSettings v-else-if="activeSection === 'appearance'" />
      <KeyboardSettings v-else-if="activeSection === 'keyboard'" />
      <VisualEffectsSettings v-else-if="activeSection === 'visualEffects'" />
    </div>
  </div>
</template>
