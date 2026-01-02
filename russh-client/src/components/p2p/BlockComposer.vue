<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { 
  Send, 
  Code, 
  FileUp, 
  Puzzle, 
  X,
  Vote,
  TextCursor,
  ToggleLeft,
  BarChart3
} from 'lucide-vue-next';
import type { WidgetConfig, WidgetType } from '@/types/blocks';

type ComposerMode = 'text' | 'code' | 'widget';

const emit = defineEmits<{
  sendText: [content: string, format: 'plain' | 'markdown'];
  sendCode: [content: string, language: string, filename?: string];
  sendFile: [file: File];
  sendWidget: [widgetType: WidgetType, config: WidgetConfig];
  typing: [isTyping: boolean];
}>();

const mode = ref<ComposerMode>('text');
const textContent = ref('');
const codeContent = ref('');
const codeLanguage = ref('javascript');
const codeFilename = ref('');
const showCommandPalette = ref(false);
const showWidgetMenu = ref(false);
const fileInputRef = ref<HTMLInputElement>();

// Widget builder state
const widgetType = ref<WidgetType>('button');
const widgetLabel = ref('');
const widgetAction = ref('');
const pollQuestion = ref('');
const pollOptions = ref(['', '']);
const confirmMessage = ref('');

const languages = [
  'javascript', 'typescript', 'python', 'rust', 'go', 
  'java', 'cpp', 'c', 'html', 'css', 'json', 'yaml', 'bash', 'sql'
];

const widgetTypes: { type: WidgetType; label: string; icon: typeof Vote }[] = [
  { type: 'button', label: 'Button', icon: ToggleLeft },
  { type: 'input', label: 'Input', icon: TextCursor },
  { type: 'poll', label: 'Poll', icon: Vote },
  { type: 'progress', label: 'Progress', icon: BarChart3 },
  { type: 'confirm', label: 'Confirm', icon: ToggleLeft },
];

const canSend = computed(() => {
  switch (mode.value) {
    case 'text':
      return textContent.value.trim().length > 0;
    case 'code':
      return codeContent.value.trim().length > 0;
    case 'widget':
      return isWidgetValid.value;
    default:
      return false;
  }
});

const isWidgetValid = computed(() => {
  switch (widgetType.value) {
    case 'button':
      return widgetLabel.value.trim() && widgetAction.value.trim();
    case 'input':
      return widgetLabel.value.trim();
    case 'poll':
      return pollQuestion.value.trim() && pollOptions.value.filter(o => o.trim()).length >= 2;
    case 'confirm':
      return confirmMessage.value.trim();
    case 'progress':
      return widgetLabel.value.trim();
    default:
      return false;
  }
});

// Watch for slash commands
watch(textContent, (val) => {
  if (val === '/') {
    showCommandPalette.value = true;
  } else if (!val.startsWith('/')) {
    showCommandPalette.value = false;
  }
  
  // Emit typing indicator
  emit('typing', val.length > 0);
});

// Auto-detect code paste
watch(textContent, (val) => {
  if (mode.value === 'text' && looksLikeCode(val)) {
    // Suggest code mode
  }
});

function looksLikeCode(text: string): boolean {
  const codePatterns = [
    /^(import|export|const|let|var|function|class|def|fn|pub|async)\s/m,
    /[{}\[\]();]/,
    /^\s{2,}/m,
  ];
  return codePatterns.some(p => p.test(text)) && text.split('\n').length > 2;
}

function selectCommand(cmd: string) {
  showCommandPalette.value = false;
  textContent.value = '';
  
  switch (cmd) {
    case 'code':
      mode.value = 'code';
      break;
    case 'file':
      fileInputRef.value?.click();
      break;
    case 'widget':
      mode.value = 'widget';
      showWidgetMenu.value = true;
      break;
  }
}

function handleSend() {
  if (!canSend.value) return;
  
  switch (mode.value) {
    case 'text':
      const isMarkdown = /[*_`\[\]]/.test(textContent.value);
      emit('sendText', textContent.value, isMarkdown ? 'markdown' : 'plain');
      textContent.value = '';
      break;
    case 'code':
      emit('sendCode', codeContent.value, codeLanguage.value, codeFilename.value || undefined);
      codeContent.value = '';
      codeFilename.value = '';
      mode.value = 'text';
      break;
    case 'widget':
      sendWidget();
      break;
  }
  
  emit('typing', false);
}

function sendWidget() {
  let config: WidgetConfig;
  
  switch (widgetType.value) {
    case 'button':
      config = {
        type: 'button',
        label: widgetLabel.value,
        action: widgetAction.value,
        variant: 'primary',
      };
      break;
    case 'input':
      config = {
        type: 'input',
        label: widgetLabel.value,
        placeholder: 'Enter your response...',
      };
      break;
    case 'poll':
      config = {
        type: 'poll',
        question: pollQuestion.value,
        options: pollOptions.value.filter(o => o.trim()),
      };
      break;
    case 'progress':
      config = {
        type: 'progress',
        label: widgetLabel.value,
        value: 0,
        max: 100,
      };
      break;
    case 'confirm':
      config = {
        type: 'confirm',
        message: confirmMessage.value,
      };
      break;
    default:
      return;
  }
  
  emit('sendWidget', widgetType.value, config);
  resetWidgetBuilder();
  mode.value = 'text';
}

function resetWidgetBuilder() {
  widgetLabel.value = '';
  widgetAction.value = '';
  pollQuestion.value = '';
  pollOptions.value = ['', ''];
  confirmMessage.value = '';
  showWidgetMenu.value = false;
}

function handleFileSelect(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (file) {
    emit('sendFile', file);
    input.value = '';
  }
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  const file = event.dataTransfer?.files[0];
  if (file) {
    emit('sendFile', file);
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    handleSend();
  }
}

function addPollOption() {
  pollOptions.value.push('');
}

function removePollOption(index: number) {
  if (pollOptions.value.length > 2) {
    pollOptions.value.splice(index, 1);
  }
}

function cancelMode() {
  mode.value = 'text';
  resetWidgetBuilder();
}
</script>

<template>
  <div 
    class="block-composer border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800"
    @drop="handleDrop"
    @dragover.prevent
  >
    <!-- Mode Indicator -->
    <div 
      v-if="mode !== 'text'"
      class="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700"
    >
      <div class="flex items-center gap-2 text-sm">
        <Code v-if="mode === 'code'" class="w-4 h-4 text-blue-500" />
        <Puzzle v-if="mode === 'widget'" class="w-4 h-4 text-purple-500" />
        <span class="font-medium text-gray-700 dark:text-gray-300">
          {{ mode === 'code' ? 'Code Block' : 'Widget Builder' }}
        </span>
      </div>
      <button 
        @click="cancelMode"
        class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
    
    <!-- Command Palette -->
    <div 
      v-if="showCommandPalette"
      class="absolute bottom-full left-4 mb-2 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 overflow-hidden z-10"
    >
      <div class="p-2 text-xs text-gray-500 border-b border-gray-200 dark:border-gray-700">
        Commands
      </div>
      <button 
        @click="selectCommand('code')"
        class="w-full flex items-center gap-3 px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-left"
      >
        <Code class="w-4 h-4 text-blue-500" />
        <div>
          <p class="text-sm font-medium text-gray-900 dark:text-white">Code Block</p>
          <p class="text-xs text-gray-500">Share syntax-highlighted code</p>
        </div>
      </button>
      <button 
        @click="selectCommand('file')"
        class="w-full flex items-center gap-3 px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-left"
      >
        <FileUp class="w-4 h-4 text-green-500" />
        <div>
          <p class="text-sm font-medium text-gray-900 dark:text-white">File</p>
          <p class="text-xs text-gray-500">Upload and share a file</p>
        </div>
      </button>
      <button 
        @click="selectCommand('widget')"
        class="w-full flex items-center gap-3 px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-left"
      >
        <Puzzle class="w-4 h-4 text-purple-500" />
        <div>
          <p class="text-sm font-medium text-gray-900 dark:text-white">Widget</p>
          <p class="text-xs text-gray-500">Create interactive elements</p>
        </div>
      </button>
    </div>
    
    <!-- Text Input -->
    <div v-if="mode === 'text'" class="p-4">
      <div class="flex items-end gap-2">
        <div class="flex-1 relative">
          <textarea
            ref="textareaRef"
            v-model="textContent"
            placeholder="Type a message... (/ for commands)"
            rows="1"
            class="w-full px-4 py-3 bg-gray-100 dark:bg-gray-700 rounded-2xl resize-none text-gray-900 dark:text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
            @keydown="handleKeydown"
          />
        </div>
        <div class="flex items-center gap-1">
          <button 
            @click="fileInputRef?.click()"
            class="p-3 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-full transition-colors"
            title="Attach file"
          >
            <FileUp class="w-5 h-5" />
          </button>
          <button 
            @click="handleSend"
            :disabled="!canSend"
            class="p-3 bg-blue-500 hover:bg-blue-600 disabled:bg-gray-300 dark:disabled:bg-gray-600 text-white rounded-full transition-colors"
          >
            <Send class="w-5 h-5" />
          </button>
        </div>
      </div>
    </div>
    
    <!-- Code Input -->
    <div v-else-if="mode === 'code'" class="p-4 space-y-3">
      <div class="flex items-center gap-2">
        <select 
          v-model="codeLanguage"
          class="px-3 py-1.5 bg-gray-100 dark:bg-gray-700 rounded-lg text-sm text-gray-900 dark:text-white border-none focus:ring-2 focus:ring-blue-500"
        >
          <option v-for="lang in languages" :key="lang" :value="lang">
            {{ lang }}
          </option>
        </select>
        <input 
          v-model="codeFilename"
          placeholder="filename (optional)"
          class="flex-1 px-3 py-1.5 bg-gray-100 dark:bg-gray-700 rounded-lg text-sm text-gray-900 dark:text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
      <textarea
        v-model="codeContent"
        placeholder="Paste or type your code here..."
        rows="6"
        class="w-full px-4 py-3 bg-gray-900 rounded-lg font-mono text-sm text-gray-100 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
        @keydown.ctrl.enter="handleSend"
      />
      <div class="flex justify-end">
        <button 
          @click="handleSend"
          :disabled="!canSend"
          class="flex items-center gap-2 px-4 py-2 bg-blue-500 hover:bg-blue-600 disabled:bg-gray-300 dark:disabled:bg-gray-600 text-white rounded-lg transition-colors"
        >
          <Send class="w-4 h-4" />
          Send Code
        </button>
      </div>
    </div>
    
    <!-- Widget Builder -->
    <div v-else-if="mode === 'widget'" class="p-4 space-y-4">
      <!-- Widget Type Selector -->
      <div class="flex gap-2 overflow-x-auto pb-2">
        <button 
          v-for="wt in widgetTypes"
          :key="wt.type"
          @click="widgetType = wt.type"
          class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm whitespace-nowrap transition-colors"
          :class="widgetType === wt.type 
            ? 'bg-purple-100 dark:bg-purple-900/50 text-purple-700 dark:text-purple-300' 
            : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'"
        >
          <component :is="wt.icon" class="w-4 h-4" />
          {{ wt.label }}
        </button>
      </div>
      
      <!-- Button Config -->
      <template v-if="widgetType === 'button'">
        <input 
          v-model="widgetLabel"
          placeholder="Button label"
          class="w-full px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500"
        />
        <input 
          v-model="widgetAction"
          placeholder="Action identifier"
          class="w-full px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500"
        />
      </template>
      
      <!-- Input Config -->
      <template v-else-if="widgetType === 'input'">
        <input 
          v-model="widgetLabel"
          placeholder="Input label / question"
          class="w-full px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500"
        />
      </template>
      
      <!-- Poll Config -->
      <template v-else-if="widgetType === 'poll'">
        <input 
          v-model="pollQuestion"
          placeholder="Poll question"
          class="w-full px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500"
        />
        <div class="space-y-2">
          <div 
            v-for="(_, i) in pollOptions"
            :key="i"
            class="flex items-center gap-2"
          >
            <input 
              v-model="pollOptions[i]"
              :placeholder="`Option ${i + 1}`"
              class="flex-1 px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500"
            />
            <button 
              v-if="pollOptions.length > 2"
              @click="removePollOption(i)"
              class="p-2 text-gray-400 hover:text-red-500"
            >
              <X class="w-4 h-4" />
            </button>
          </div>
          <button 
            @click="addPollOption"
            class="text-sm text-purple-500 hover:text-purple-600"
          >
            + Add option
          </button>
        </div>
      </template>
      
      <!-- Confirm Config -->
      <template v-else-if="widgetType === 'confirm'">
        <input 
          v-model="confirmMessage"
          placeholder="Confirmation message"
          class="w-full px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500"
        />
      </template>
      
      <!-- Progress Config -->
      <template v-else-if="widgetType === 'progress'">
        <input 
          v-model="widgetLabel"
          placeholder="Progress label"
          class="w-full px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg text-gray-900 dark:text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500"
        />
      </template>
      
      <div class="flex justify-end">
        <button 
          @click="handleSend"
          :disabled="!canSend"
          class="flex items-center gap-2 px-4 py-2 bg-purple-500 hover:bg-purple-600 disabled:bg-gray-300 dark:disabled:bg-gray-600 text-white rounded-lg transition-colors"
        >
          <Puzzle class="w-4 h-4" />
          Send Widget
        </button>
      </div>
    </div>
    
    <!-- Hidden file input -->
    <input 
      ref="fileInputRef"
      type="file"
      class="hidden"
      @change="handleFileSelect"
    />
  </div>
</template>
