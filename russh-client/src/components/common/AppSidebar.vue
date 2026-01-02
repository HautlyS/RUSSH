<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { useP2P } from '@/composables/useP2P';
import { useVisualEffects } from '@/composables/useVisualEffects';
import { usePlatform } from '@/composables/usePlatform';
import { Plus, Search, Star, Folder, Server, Users, ChevronDown, ChevronRight } from 'lucide-vue-next';
import ConnectionItem from '@/components/connections/ConnectionItem.vue';
import GradualBlur from '@/components/extra/GradualBlur.vue';
import FlowingMenu from '@/components/extra/FlowingMenu.vue';

defineProps<{
  collapsed: boolean;
}>();

const router = useRouter();
const connectionStore = useConnectionStore();
const { peers, isOnline } = useP2P();
const { isGradualBlurEnabled, isFlowingMenuEnabled } = useVisualEffects();
const { isMobile } = usePlatform();

// Mobile menu items for FlowingMenu
const mobileMenuItems = computed(() => [
  { link: '/dashboard', text: 'Dashboard', image: '/icons/dashboard.png' },
  { link: '/connections', text: 'Connections', image: '/icons/connections.png' },
  { link: '/p2p', text: 'P2P', image: '/icons/p2p.png' },
  { link: '/settings', text: 'Settings', image: '/icons/settings.png' },
]);

// Show flowing menu on mobile when enabled
const showFlowingMenu = computed(() => 
  isFlowingMenuEnabled.value && isMobile.value
);

const searchQuery = ref('');
const expandedFolders = ref<Set<string>>(new Set());

const favoriteProfiles = computed(() => 
  connectionStore.profiles.filter(p => p.tags.includes('favorite'))
);

const folders = computed(() => connectionStore.folders);

const ungroupedProfiles = computed(() => 
  connectionStore.profiles.filter(p => !p.folder && !p.tags.includes('favorite'))
);

const filteredProfiles = computed(() => {
  if (!searchQuery.value) return connectionStore.profiles;
  const query = searchQuery.value.toLowerCase();
  return connectionStore.profiles.filter(p => 
    p.name.toLowerCase().includes(query) ||
    p.host.toLowerCase().includes(query)
  );
});

function getProfilesByFolder(folder: string) {
  return connectionStore.profiles.filter(p => p.folder === folder);
}

function toggleFolder(folder: string) {
  if (expandedFolders.value.has(folder)) {
    expandedFolders.value.delete(folder);
  } else {
    expandedFolders.value.add(folder);
  }
}

function isFolderExpanded(folder: string) {
  return expandedFolders.value.has(folder);
}

function getPeerId(peer: { peerId?: string }): string {
  return peer.peerId || 'unknown';
}
</script>

<template>
  <!-- Mobile FlowingMenu -->
  <aside 
    v-if="showFlowingMenu"
    class="h-full bg-gray-900"
  >
    <FlowingMenu :items="mobileMenuItems" />
  </aside>
  
  <!-- Desktop Sidebar -->
  <aside 
    v-else
    :class="[
      'h-full bg-gray-50 dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 flex flex-col transition-all duration-200 relative',
      collapsed ? 'w-16' : 'w-64'
    ]"
  >
    <!-- Quick Actions -->
    <div class="p-3 border-b border-gray-200 dark:border-gray-700">
      <button 
        @click="router.push('/connections/new')"
        class="w-full flex items-center justify-center gap-2 px-3 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
      >
        <Plus class="w-4 h-4" />
        <span v-if="!collapsed">New Connection</span>
      </button>
    </div>
    
    <!-- Search -->
    <div v-if="!collapsed" class="p-3">
      <div class="relative">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input 
          v-model="searchQuery"
          type="text"
          placeholder="Filter connections..."
          class="w-full pl-9 pr-3 py-1.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>
    </div>
    
    <!-- Connection List -->
    <nav class="flex-1 overflow-y-auto p-2 space-y-4 relative">
      <!-- GradualBlur at top of scroll area -->
      <GradualBlur 
        v-if="isGradualBlurEnabled && !collapsed"
        position="top"
        :height="'2rem'"
        :strength="1.5"
        :div-count="3"
        :z-index="10"
      />
      <!-- Favorites -->
      <div v-if="favoriteProfiles.length > 0">
        <div v-if="!collapsed" class="flex items-center gap-2 px-2 py-1 text-xs font-medium text-gray-500 uppercase">
          <Star class="w-3 h-3" />
          Favorites
        </div>
        <div class="space-y-1">
          <ConnectionItem 
            v-for="profile in favoriteProfiles" 
            :key="profile.id"
            :profile="profile"
            :collapsed="collapsed"
          />
        </div>
      </div>
      
      <!-- Folders -->
      <div v-for="folder in folders" :key="folder">
        <button 
          v-if="!collapsed"
          @click="toggleFolder(folder)"
          class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-gray-500 uppercase hover:text-gray-700 dark:hover:text-gray-300"
        >
          <component :is="isFolderExpanded(folder) ? ChevronDown : ChevronRight" class="w-3 h-3" />
          <Folder class="w-3 h-3" />
          {{ folder }}
        </button>
        <div v-if="isFolderExpanded(folder) || collapsed" class="space-y-1 mt-1">
          <ConnectionItem 
            v-for="profile in getProfilesByFolder(folder)" 
            :key="profile.id"
            :profile="profile"
            :collapsed="collapsed"
          />
        </div>
      </div>
      
      <!-- Ungrouped -->
      <div v-if="ungroupedProfiles.length > 0">
        <div v-if="!collapsed" class="flex items-center gap-2 px-2 py-1 text-xs font-medium text-gray-500 uppercase">
          <Server class="w-3 h-3" />
          Connections
        </div>
        <div class="space-y-1">
          <ConnectionItem 
            v-for="profile in ungroupedProfiles" 
            :key="profile.id"
            :profile="profile"
            :collapsed="collapsed"
          />
        </div>
      </div>
      
      <!-- P2P Peers -->
      <div v-if="peers.length > 0">
        <div v-if="!collapsed" class="flex items-center gap-2 px-2 py-1 text-xs font-medium text-gray-500 uppercase">
          <Users class="w-3 h-3" />
          P2P Peers
        </div>
        <div class="space-y-1">
          <div 
            v-for="peer in peers" 
            :key="getPeerId(peer)"
            class="flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 cursor-pointer"
          >
            <div class="w-2 h-2 rounded-full bg-green-500"></div>
            <span v-if="!collapsed" class="text-sm truncate">{{ getPeerId(peer).slice(0, 8) }}...</span>
          </div>
        </div>
      </div>
      
      <!-- GradualBlur at bottom of scroll area -->
      <GradualBlur 
        v-if="isGradualBlurEnabled && !collapsed"
        position="bottom"
        :height="'2rem'"
        :strength="1.5"
        :div-count="3"
        :z-index="10"
      />
    </nav>
    
    <!-- Footer -->
    <div class="p-3 border-t border-gray-200 dark:border-gray-700">
      <div class="flex items-center gap-2 text-sm text-gray-500">
        <div 
          class="w-2 h-2 rounded-full"
          :class="isOnline ? 'bg-green-500' : 'bg-gray-400'"
        ></div>
        <span v-if="!collapsed">{{ isOnline ? 'P2P Online' : 'P2P Offline' }}</span>
      </div>
    </div>
  </aside>
</template>
