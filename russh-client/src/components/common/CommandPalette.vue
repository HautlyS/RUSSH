<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { useTheme } from '@/composables/useTheme';
import { useVisualEffects } from '@/composables/useVisualEffects';
import { Search, Plus, Zap, Settings, Moon, Sun, Keyboard, Terminal, Server } from 'lucide-vue-next';
import GradualBlur from '@/components/extra/GradualBlur.vue';

const router = useRouter();
const connectionStore = useConnectionStore();
const { isDark, toggleTheme } = useTheme();
const { isGradualBlurEnabled } = useVisualEffects();

const isOpen = ref(false);
const query = ref('');
const selectedIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);

// Define available actions
const actions = [
  { id: 'new-connection', label: 'New Connection', icon: Plus, action: () => router.push('/connections/new') },
  { id: 'quick-connect', label: 'Quick Connect', icon: Zap, action: () => router.push('/connections/quick') },
  { id: 'settings', label: 'Open Settings', icon: Settings, action: () => router.push('/settings') },
  { id: 'toggle-theme', label: 'Toggle Theme', icon: isDark.value ? Sun : Moon, action: () => toggleTheme() },
  { id: 'shortcuts', label: 'Keyboard Shortcuts', icon: Keyboard, action: () => showShortcuts() },
  { id: 'p2p', label: 'P2P Connections', icon: Server, action: () => router.push('/p2p') },
];

const filteredActions = computed(() => {
  if (!query.value) return actions.slice(0, 5);
  const q = query.value.toLowerCase();
  return actions.filter(a => a.label.toLowerCase().includes(q));
});

const filteredConnections = computed(() => {
  if (!query.value) return [];
  const q = query.value.toLowerCase();
  return connectionStore.profiles
    .filter(p => 
      p.name.toLowerCase().includes(q) ||
      p.host.toLowerCase().includes(q)
    )
    .slice(0, 5)
    .map(p => ({
      id: p.id,
      label: p.name,
      sublabel: `${p.username}@${p.host}`,
      icon: Terminal,
      action: () => connectionStore.connect(p.id),
    }));
});

const allItems = computed(() => [
  ...filteredActions.value,
  ...filteredConnections.value,
]);

const hasResults = computed(() => allItems.value.length > 0);

function open() {
  isOpen.value = true;
  query.value = '';
  selectedIndex.value = 0;
  nextTick(() => inputRef.value?.focus());
}

function close() {
  isOpen.value = false;
}

function handleKeydown(event: KeyboardEvent) {
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      selectedIndex.value = (selectedIndex.value + 1) % allItems.value.length;
      break;
    case 'ArrowUp':
      event.preventDefault();
      selectedIndex.value = (selectedIndex.value - 1 + allItems.value.length) % allItems.value.length;
      break;
    case 'Enter':
      event.preventDefault();
      executeSelectedItem();
      break;
    case 'Escape':
      close();
      break;
  }
}

function executeItem(item: typeof allItems.value[0]) {
  item.action();
  close();
}

function executeSelectedItem() {
  const item = allItems.value[selectedIndex.value];
  if (item) executeItem(item);
}

function showShortcuts() {
  // TODO: Show shortcuts modal
  router.push('/settings?tab=keyboard');
}

// Reset selection when query changes
watch(query, () => {
  selectedIndex.value = 0;
});

// Listen for open event
onMounted(() => {
  document.addEventListener('open-command-palette', open);
});

onUnmounted(() => {
  document.removeEventListener('open-command-palette', open);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div 
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]"
        @click.self="close"
      >
        <!-- Backdrop with GradualBlur -->
        <div class="absolute inset-0 bg-black/50">
          <GradualBlur 
            v-if="isGradualBlurEnabled"
            position="top"
            :height="'100%'"
            :strength="3"
            :div-count="8"
            :opacity="0.8"
            curve="ease-out"
            target="parent"
          />
        </div>
        
        <div 
          class="relative w-full max-w-lg bg-white dark:bg-gray-800 rounded-xl shadow-2xl overflow-hidden animate-slide-up"
          @keydown="handleKeydown"
        >
          <!-- Search Input -->
          <div class="p-4 border-b border-gray-200 dark:border-gray-700">
            <div class="flex items-center gap-3">
              <Search class="w-5 h-5 text-gray-400" />
              <input 
                ref="inputRef"
                v-model="query"
                type="text"
                placeholder="Type a command or search..."
                class="flex-1 bg-transparent outline-none text-lg"
              />
              <kbd class="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-xs">ESC</kbd>
            </div>
          </div>
          
          <!-- Results -->
          <div class="max-h-80 overflow-y-auto">
            <!-- Actions -->
            <div v-if="filteredActions.length" class="p-2">
              <div class="px-3 py-1 text-xs font-medium text-gray-500 uppercase">Actions</div>
              <button 
                v-for="(item, index) in filteredActions" 
                :key="item.id"
                @click="executeItem(item)"
                :class="[
                  'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left transition-colors',
                  selectedIndex === index 
                    ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400' 
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700'
                ]"
              >
                <component :is="item.icon" class="w-4 h-4" />
                <span>{{ item.label }}</span>
              </button>
            </div>
            
            <!-- Connections -->
            <div v-if="filteredConnections.length" class="p-2 border-t border-gray-200 dark:border-gray-700">
              <div class="px-3 py-1 text-xs font-medium text-gray-500 uppercase">Connections</div>
              <button 
                v-for="(item, index) in filteredConnections" 
                :key="item.id"
                @click="executeItem(item)"
                :class="[
                  'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left transition-colors',
                  selectedIndex === filteredActions.length + index 
                    ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400' 
                    : 'hover:bg-gray-100 dark:hover:bg-gray-700'
                ]"
              >
                <component :is="item.icon" class="w-4 h-4" />
                <div class="flex-1 min-w-0">
                  <div class="truncate">{{ item.label }}</div>
                  <div class="text-xs text-gray-500 truncate">{{ item.sublabel }}</div>
                </div>
              </button>
            </div>
            
            <!-- No Results -->
            <div v-if="!hasResults && query" class="p-8 text-center text-gray-500">
              No results found for "{{ query }}"
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
