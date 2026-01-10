<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { useTheme } from '@/composables/useTheme';
import { 
  Search, Plus, Zap, Settings, Moon, Sun, Keyboard, 
  Terminal, Server, ArrowRight, Command
} from 'lucide-vue-next';

const router = useRouter();
const connectionStore = useConnectionStore();
const { isDark, toggleTheme } = useTheme();

const isOpen = ref(false);
const query = ref('');
const selectedIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);

const actions = [
  { id: 'new-connection', label: 'New Connection', desc: 'Create SSH profile', icon: Plus, action: () => router.push('/connections/new') },
  { id: 'quick-connect', label: 'Quick Connect', desc: 'One-time session', icon: Zap, action: () => router.push('/connections?quick=true') },
  { id: 'settings', label: 'Settings', desc: 'App preferences', icon: Settings, action: () => router.push('/settings') },
  { id: 'toggle-theme', label: 'Toggle Theme', desc: isDark.value ? 'Switch to light' : 'Switch to dark', icon: isDark.value ? Sun : Moon, action: () => toggleTheme() },
  { id: 'shortcuts', label: 'Keyboard Shortcuts', desc: 'View all shortcuts', icon: Keyboard, action: () => router.push('/settings?tab=keyboard') },
  { id: 'p2p', label: 'P2P Network', desc: 'Connect via NAT', icon: Server, action: () => router.push('/p2p') },
];

const filteredActions = computed(() => {
  if (!query.value) return actions.slice(0, 5);
  const q = query.value.toLowerCase();
  return actions.filter(a => 
    a.label.toLowerCase().includes(q) || 
    a.desc.toLowerCase().includes(q)
  );
});

const filteredConnections = computed(() => {
  if (!query.value) return [];
  const q = query.value.toLowerCase();
  return connectionStore.profiles
    .filter(p => 
      p.name.toLowerCase().includes(q) ||
      p.host.toLowerCase().includes(q) ||
      p.username.toLowerCase().includes(q)
    )
    .slice(0, 5)
    .map(p => ({
      id: p.id,
      label: p.name,
      desc: `${p.username}@${p.host}`,
      icon: Terminal,
      color: p.color,
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

watch(query, () => {
  selectedIndex.value = 0;
});

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
        class="fixed inset-0 z-[100] flex items-start justify-center pt-[15vh]"
        @click.self="close"
      >
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" />
        
        <!-- Modal -->
        <div 
          class="relative w-full max-w-xl mx-4 glass-card overflow-hidden animate-scale-in"
          style="background: rgba(15, 15, 20, 0.95); border-color: rgba(255,255,255,0.1);"
          @keydown="handleKeydown"
        >
          <!-- Search Input -->
          <div class="p-4 border-b border-white/5">
            <div class="flex items-center gap-3">
              <Search class="w-5 h-5 text-gray-500" />
              <input 
                ref="inputRef"
                v-model="query"
                type="text"
                placeholder="Search commands, connections..."
                class="flex-1 bg-transparent outline-none text-base text-white placeholder-gray-500"
              />
              <div class="flex items-center gap-1">
                <kbd class="px-1.5 py-0.5 bg-white/5 border border-white/10 rounded text-xs text-gray-500">
                  ESC
                </kbd>
              </div>
            </div>
          </div>
          
          <!-- Results -->
          <div class="max-h-[50vh] overflow-y-auto scrollbar-hide">
            <!-- Actions -->
            <div v-if="filteredActions.length" class="p-2">
              <div class="px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">
                Actions
              </div>
              <button 
                v-for="(item, index) in filteredActions" 
                :key="item.id"
                @click="executeItem(item)"
                @mouseenter="selectedIndex = index"
                :class="[
                  'w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-left transition-all group',
                  selectedIndex === index 
                    ? 'bg-green-500/10 border border-green-500/20' 
                    : 'border border-transparent hover:bg-white/5'
                ]"
              >
                <div 
                  :class="[
                    'w-9 h-9 rounded-lg flex items-center justify-center transition-colors',
                    selectedIndex === index ? 'bg-green-500/20' : 'bg-white/5'
                  ]"
                >
                  <component 
                    :is="item.icon" 
                    :class="[
                      'w-4 h-4',
                      selectedIndex === index ? 'text-green-400' : 'text-gray-400'
                    ]"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <div :class="selectedIndex === index ? 'text-green-400' : 'text-white'">
                    {{ item.label }}
                  </div>
                  <div class="text-xs text-gray-500">{{ item.desc }}</div>
                </div>
                <ArrowRight 
                  :class="[
                    'w-4 h-4 transition-all',
                    selectedIndex === index 
                      ? 'text-green-400 translate-x-0 opacity-100' 
                      : 'text-gray-600 -translate-x-2 opacity-0'
                  ]"
                />
              </button>
            </div>
            
            <!-- Connections -->
            <div v-if="filteredConnections.length" class="p-2 border-t border-white/5">
              <div class="px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">
                Connections
              </div>
              <button 
                v-for="(item, index) in filteredConnections" 
                :key="item.id"
                @click="executeItem(item)"
                @mouseenter="selectedIndex = filteredActions.length + index"
                :class="[
                  'w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-left transition-all group',
                  selectedIndex === filteredActions.length + index 
                    ? 'bg-green-500/10 border border-green-500/20' 
                    : 'border border-transparent hover:bg-white/5'
                ]"
              >
                <div 
                  class="w-9 h-9 rounded-lg flex items-center justify-center"
                  :style="{ backgroundColor: (item.color || '#6b7280') + '20' }"
                >
                  <component 
                    :is="item.icon" 
                    class="w-4 h-4"
                    :style="{ color: item.color || '#6b7280' }"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <div 
                    :class="selectedIndex === filteredActions.length + index ? 'text-green-400' : 'text-white'"
                  >
                    {{ item.label }}
                  </div>
                  <div class="text-xs text-gray-500 truncate">{{ item.desc }}</div>
                </div>
                <ArrowRight 
                  :class="[
                    'w-4 h-4 transition-all',
                    selectedIndex === filteredActions.length + index 
                      ? 'text-green-400 translate-x-0 opacity-100' 
                      : 'text-gray-600 -translate-x-2 opacity-0'
                  ]"
                />
              </button>
            </div>
            
            <!-- No Results -->
            <div v-if="!hasResults && query" class="p-8 text-center">
              <div class="text-gray-500 mb-2">No results for "{{ query }}"</div>
              <div class="text-xs text-gray-600">Try searching for connections or commands</div>
            </div>
          </div>
          
          <!-- Footer -->
          <div class="px-4 py-3 border-t border-white/5 flex items-center justify-between text-xs text-gray-500">
            <div class="flex items-center gap-4">
              <span class="flex items-center gap-1">
                <kbd class="px-1 py-0.5 bg-white/5 rounded">↑↓</kbd>
                Navigate
              </span>
              <span class="flex items-center gap-1">
                <kbd class="px-1 py-0.5 bg-white/5 rounded">↵</kbd>
                Select
              </span>
            </div>
            <div class="flex items-center gap-1">
              <Command class="w-3 h-3" />
              <span>K to open</span>
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
