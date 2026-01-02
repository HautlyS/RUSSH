<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { Plus } from 'lucide-vue-next';
import { useTerminalStore } from '@/stores/terminals';
import { useTerminal } from '@/composables/useTerminal';
import { useSettingsStore } from '@/stores/settings';
import { useVisualEffects } from '@/composables/useVisualEffects';
import TerminalTab from './TerminalTab.vue';
import TerminalToolbar from './TerminalToolbar.vue';
import TerminalSettings from './TerminalSettings.vue';
import Lightning from '@/components/extra/Lightning.vue';

const props = defineProps<{
  sessionId?: string;
}>();

const terminalStore = useTerminalStore();
const settingsStore = useSettingsStore();
const { isLightningEnabledFor, visualEffects } = useVisualEffects();

const isLightningEnabled = isLightningEnabledFor('terminal');

const terminalRef = ref<HTMLDivElement>();
const showSettings = ref(false);

const { 
  terminal, 
  initTerminal, 
  destroyTerminal, 
  clear, 
  focus,
  copySelection 
} = useTerminal();

const activeTab = computed(() => terminalStore.activeTab);
const tabs = computed(() => terminalStore.tabs);

// Initialize terminal when component mounts or active tab changes
watch([() => activeTab.value, terminalRef], async ([tab, el]) => {
  if (tab && el) {
    await nextTick();
    initTerminal(el, {
      fontSize: settingsStore.settings.terminal.fontSize,
      fontFamily: settingsStore.settings.terminal.fontFamily,
      theme: settingsStore.settings.terminal.theme,
      cursorStyle: settingsStore.settings.terminal.cursorStyle as 'block' | 'underline' | 'bar',
      cursorBlink: settingsStore.settings.terminal.cursorBlink,
      scrollback: settingsStore.settings.terminal.scrollback,
    });
  }
}, { immediate: true });

// Create initial tab if session provided
onMounted(() => {
  if (props.sessionId && tabs.value.length === 0) {
    terminalStore.createTab(props.sessionId, props.sessionId, 'Terminal');
  }
});

onUnmounted(() => {
  destroyTerminal();
});

function handleNewTab() {
  if (props.sessionId) {
    terminalStore.createTab(props.sessionId, props.sessionId, `Terminal ${tabs.value.length + 1}`);
  }
}

function handleCloseTab(tabId: string) {
  terminalStore.closeTab(tabId);
}

function handleSelectTab(tabId: string) {
  terminalStore.setActiveTab(tabId);
}

function handleClear() {
  clear();
}

function handleCopy() {
  copySelection();
}

function handleExport() {
  // Export terminal content
  const content = terminal.value?.buffer.active;
  if (content) {
    const lines: string[] = [];
    for (let i = 0; i < content.length; i++) {
      const line = content.getLine(i);
      if (line) lines.push(line.translateToString());
    }
    const blob = new Blob([lines.join('\n')], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `terminal-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  }
}

function handleFullscreen() {
  const el = terminalRef.value?.parentElement;
  if (el) {
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      el.requestFullscreen();
    }
  }
}

function handleSettingsApply() {
  showSettings.value = false;
  // Reinitialize terminal with new settings
  if (terminalRef.value) {
    destroyTerminal();
    initTerminal(terminalRef.value, {
      fontSize: settingsStore.settings.terminal.fontSize,
      fontFamily: settingsStore.settings.terminal.fontFamily,
      theme: settingsStore.settings.terminal.theme,
      cursorStyle: settingsStore.settings.terminal.cursorStyle as 'block' | 'underline' | 'bar',
      cursorBlink: settingsStore.settings.terminal.cursorBlink,
      scrollback: settingsStore.settings.terminal.scrollback,
    });
  }
}
</script>

<template>
  <div class="terminal-container flex flex-col h-full bg-gray-900 relative">
    <!-- Lightning Background Effect -->
    <div 
      v-if="isLightningEnabled" 
      class="absolute inset-0 z-0 opacity-30"
    >
      <Lightning
        :hue="visualEffects.lightning.hue"
        :intensity="visualEffects.lightning.intensity"
        :speed="visualEffects.lightning.speed"
        :size="1"
      />
    </div>

    <!-- Tab Bar -->
    <div class="relative z-10 flex items-center bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
      <div class="flex-1 flex items-center overflow-x-auto">
        <TerminalTab
          v-for="tab in tabs"
          :key="tab.id"
          :id="tab.id"
          :title="tab.title"
          :active="tab.id === activeTab?.id"
          @select="handleSelectTab(tab.id)"
          @close="handleCloseTab(tab.id)"
        />
      </div>
      <button
        @click="handleNewTab"
        class="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
        title="New tab"
      >
        <Plus class="w-4 h-4" />
      </button>
    </div>
    
    <!-- Toolbar -->
    <TerminalToolbar
      class="relative z-10"
      @clear="handleClear"
      @search="() => {}"
      @settings="showSettings = true"
      @split="() => {}"
      @copy="handleCopy"
      @export="handleExport"
      @fullscreen="handleFullscreen"
    />
    
    <!-- Terminal Area -->
    <div 
      ref="terminalRef" 
      class="flex-1 p-2 relative z-10"
      @click="focus"
    />
    
    <!-- Settings Modal -->
    <TerminalSettings
      :visible="showSettings"
      @close="showSettings = false"
      @apply="handleSettingsApply"
    />
  </div>
</template>

<style scoped>
.terminal-container :deep(.xterm) {
  height: 100%;
}
.terminal-container :deep(.xterm-viewport) {
  overflow-y: auto !important;
}
</style>
