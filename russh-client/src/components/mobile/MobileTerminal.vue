<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { useTerminal } from '@/composables/useTerminal';
import { useSettingsStore } from '@/stores/settings';
import { usePlatform } from '@/composables/usePlatform';
import { 
  ArrowUp, ArrowDown, ArrowLeft, ArrowRight, 
  CornerDownLeft, Delete, Keyboard, X, Copy, Clipboard
} from 'lucide-vue-next';

const props = defineProps<{
  sessionId: string;
}>();

const settingsStore = useSettingsStore();
const { hapticFeedback } = usePlatform();
const { terminal, initTerminal, sendInput, resize } = useTerminal(props.sessionId);

const terminalRef = ref<HTMLDivElement>();
const showSpecialKeys = ref(true);
const isKeyboardVisible = ref(false);

// Special keys for mobile
const specialKeys = [
  { key: 'Tab', label: 'Tab', code: '\t' },
  { key: 'Esc', label: 'Esc', code: '\x1b' },
  { key: 'Ctrl', label: 'Ctrl', modifier: true },
  { key: 'Alt', label: 'Alt', modifier: true },
];

const ctrlActive = ref(false);
const altActive = ref(false);

// Arrow keys
const arrowKeys = [
  { icon: ArrowUp, code: '\x1b[A' },
  { icon: ArrowDown, code: '\x1b[B' },
  { icon: ArrowLeft, code: '\x1b[D' },
  { icon: ArrowRight, code: '\x1b[C' },
];

function handleSpecialKey(key: typeof specialKeys[0]) {
  hapticFeedback('light');
  
  if (key.modifier) {
    if (key.key === 'Ctrl') ctrlActive.value = !ctrlActive.value;
    if (key.key === 'Alt') altActive.value = !altActive.value;
    return;
  }
  
  let code = key.code;
  if (ctrlActive.value && key.code.length === 1) {
    code = String.fromCharCode(key.code.charCodeAt(0) & 0x1f);
    ctrlActive.value = false;
  }
  
  sendInput(code);
}

function handleArrowKey(code: string) {
  hapticFeedback('light');
  sendInput(code);
}

function handleEnter() {
  hapticFeedback('medium');
  sendInput('\r');
}

function handleBackspace() {
  hapticFeedback('light');
  sendInput('\x7f');
}

async function handleCopy() {
  hapticFeedback('light');
  const selection = terminal.value?.getSelection();
  if (selection) {
    await navigator.clipboard.writeText(selection);
  }
}

async function handlePaste() {
  hapticFeedback('light');
  const text = await navigator.clipboard.readText();
  if (text) sendInput(text);
}

function toggleKeyboard() {
  showSpecialKeys.value = !showSpecialKeys.value;
  hapticFeedback('light');
}

// Handle keyboard visibility
function onKeyboardShow() {
  isKeyboardVisible.value = true;
}

function onKeyboardHide() {
  isKeyboardVisible.value = false;
}

onMounted(() => {
  if (terminalRef.value) {
    initTerminal(terminalRef.value);
  }
  
  window.addEventListener('keyboardDidShow', onKeyboardShow);
  window.addEventListener('keyboardDidHide', onKeyboardHide);
});

onUnmounted(() => {
  window.removeEventListener('keyboardDidShow', onKeyboardShow);
  window.removeEventListener('keyboardDidHide', onKeyboardHide);
});

// Handle resize
watch(() => terminalRef.value?.clientWidth, () => {
  resize();
});
</script>

<template>
  <div class="mobile-terminal flex flex-col h-full bg-gray-900">
    <!-- Terminal Area -->
    <div 
      ref="terminalRef" 
      class="flex-1 overflow-hidden"
      :class="{ 'pb-32': showSpecialKeys && !isKeyboardVisible }"
    />
    
    <!-- Quick Actions Bar -->
    <div 
      v-if="showSpecialKeys && !isKeyboardVisible"
      class="fixed bottom-16 left-0 right-0 bg-gray-800 border-t border-gray-700 safe-area-bottom"
    >
      <!-- Clipboard Actions -->
      <div class="flex items-center justify-between px-2 py-1 border-b border-gray-700">
        <button
          @click="handleCopy"
          class="flex items-center gap-1 px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 rounded"
        >
          <Copy class="w-4 h-4" />
          Copy
        </button>
        <button
          @click="handlePaste"
          class="flex items-center gap-1 px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 rounded"
        >
          <Clipboard class="w-4 h-4" />
          Paste
        </button>
        <button
          @click="toggleKeyboard"
          class="flex items-center gap-1 px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 rounded"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
      
      <!-- Special Keys Row -->
      <div class="flex items-center gap-1 px-2 py-2">
        <button
          v-for="key in specialKeys"
          :key="key.key"
          @click="handleSpecialKey(key)"
          class="px-3 py-2 text-sm font-medium rounded transition-colors"
          :class="[
            key.modifier && ((key.key === 'Ctrl' && ctrlActive) || (key.key === 'Alt' && altActive))
              ? 'bg-blue-600 text-white'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
          ]"
        >
          {{ key.label }}
        </button>
        
        <div class="flex-1" />
        
        <!-- Arrow Keys -->
        <div class="flex items-center gap-1">
          <button
            v-for="arrow in arrowKeys"
            :key="arrow.code"
            @click="handleArrowKey(arrow.code)"
            class="p-2 bg-gray-700 text-gray-300 rounded hover:bg-gray-600"
          >
            <component :is="arrow.icon" class="w-4 h-4" />
          </button>
        </div>
      </div>
      
      <!-- Enter/Backspace Row -->
      <div class="flex items-center gap-2 px-2 pb-2">
        <button
          @click="handleBackspace"
          class="flex-1 flex items-center justify-center gap-2 py-2 bg-gray-700 text-gray-300 rounded hover:bg-gray-600"
        >
          <Delete class="w-4 h-4" />
          Delete
        </button>
        <button
          @click="handleEnter"
          class="flex-1 flex items-center justify-center gap-2 py-2 bg-green-600 text-white rounded hover:bg-green-500"
        >
          <CornerDownLeft class="w-4 h-4" />
          Enter
        </button>
      </div>
    </div>
    
    <!-- Toggle Button (when keyboard hidden) -->
    <button
      v-if="!showSpecialKeys && !isKeyboardVisible"
      @click="toggleKeyboard"
      class="fixed bottom-20 right-4 p-3 bg-blue-600 text-white rounded-full shadow-lg"
    >
      <Keyboard class="w-5 h-5" />
    </button>
  </div>
</template>

<style scoped>
.safe-area-bottom {
  padding-bottom: env(safe-area-inset-bottom, 0);
}
</style>
