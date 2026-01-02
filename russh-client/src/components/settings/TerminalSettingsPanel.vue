<script setup lang="ts">
import { computed } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { useVisualEffects } from '@/composables/useVisualEffects';
import { terminalThemes } from '@/utils/terminalThemes';
import ElasticSlider from '@/components/extra/ElasticSlider.vue';
import { Minus, Plus } from 'lucide-vue-next';

const settingsStore = useSettingsStore();
const { isElasticSliderEnabled } = useVisualEffects();
const terminal = computed(() => settingsStore.settings.terminal);

const fontFamilies = ['JetBrains Mono', 'Fira Code', 'Source Code Pro', 'Cascadia Code', 'Consolas', 'Monaco', 'monospace'];
const cursorStyles = ['block', 'underline', 'bar'];

function updateTerminal(key: string, value: any) {
  settingsStore.updateSettings({ terminal: { ...terminal.value, [key]: value } });
}
</script>

<template>
  <div class="terminal-settings space-y-6">
    <div>
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Terminal Settings</h2>
    </div>
    
    <!-- Font -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Font</h3>
      
      <div>
        <label class="block text-sm font-medium text-gray-900 dark:text-white mb-1">Font Family</label>
        <select
          :value="terminal.fontFamily"
          @change="updateTerminal('fontFamily', ($event.target as HTMLSelectElement).value)"
          class="w-full px-3 py-2 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg"
        >
          <option v-for="font in fontFamilies" :key="font" :value="font">{{ font }}</option>
        </select>
      </div>
      
      <div>
        <label class="block text-sm font-medium text-gray-900 dark:text-white mb-3">Font Size: {{ terminal.fontSize }}px</label>
        <div v-if="isElasticSliderEnabled" class="flex justify-center">
          <ElasticSlider
            :default-value="terminal.fontSize"
            :starting-value="10"
            :max-value="24"
            :is-stepped="true"
            :step-size="1"
            @update:model-value="(v: number) => updateTerminal('fontSize', v)"
          >
            <template #left-icon>
              <Minus class="w-4 h-4 text-gray-400" />
            </template>
            <template #right-icon>
              <Plus class="w-4 h-4 text-gray-400" />
            </template>
          </ElasticSlider>
        </div>
        <input 
          v-else
          type="range" 
          :value="terminal.fontSize" 
          @input="updateTerminal('fontSize', +($event.target as HTMLInputElement).value)" 
          min="10" 
          max="24" 
          class="w-full" 
        />
      </div>
    </section>
    
    <!-- Theme -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Theme</h3>
      
      <div class="grid grid-cols-3 gap-2">
        <button
          v-for="(_, name) in terminalThemes"
          :key="name"
          @click="updateTerminal('theme', name)"
          class="p-3 rounded-lg border-2 transition-colors"
          :class="terminal.theme === name ? 'border-blue-500' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300'"
        >
          <span class="text-sm capitalize">{{ name }}</span>
        </button>
      </div>
    </section>
    
    <!-- Cursor -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Cursor</h3>
      
      <div>
        <label class="block text-sm font-medium text-gray-900 dark:text-white mb-1">Cursor Style</label>
        <select
          :value="terminal.cursorStyle"
          @change="updateTerminal('cursorStyle', ($event.target as HTMLSelectElement).value)"
          class="w-full px-3 py-2 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg"
        >
          <option v-for="style in cursorStyles" :key="style" :value="style">{{ style }}</option>
        </select>
      </div>
      
      <div class="flex items-center justify-between">
        <label class="text-sm font-medium text-gray-900 dark:text-white">Cursor Blink</label>
        <button
          @click="updateTerminal('cursorBlink', !terminal.cursorBlink)"
          class="relative w-11 h-6 rounded-full transition-colors"
          :class="terminal.cursorBlink ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
        >
          <span class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform" :class="{ 'translate-x-5': terminal.cursorBlink }" />
        </button>
      </div>
    </section>
    
    <!-- Scrollback -->
    <section class="space-y-4">
      <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">Buffer</h3>
      
      <div>
        <label class="block text-sm font-medium text-gray-900 dark:text-white mb-3">Scrollback Lines: {{ terminal.scrollback?.toLocaleString() }}</label>
        <div v-if="isElasticSliderEnabled" class="flex justify-center">
          <ElasticSlider
            :default-value="terminal.scrollback"
            :starting-value="1000"
            :max-value="100000"
            :is-stepped="true"
            :step-size="1000"
            @update:model-value="(v: number) => updateTerminal('scrollback', v)"
          >
            <template #left-icon>
              <Minus class="w-4 h-4 text-gray-400" />
            </template>
            <template #right-icon>
              <Plus class="w-4 h-4 text-gray-400" />
            </template>
          </ElasticSlider>
        </div>
        <input 
          v-else
          type="range" 
          :value="terminal.scrollback" 
          @input="updateTerminal('scrollback', +($event.target as HTMLInputElement).value)" 
          min="1000" 
          max="100000" 
          step="1000" 
          class="w-full" 
        />
      </div>
    </section>
  </div>
</template>
