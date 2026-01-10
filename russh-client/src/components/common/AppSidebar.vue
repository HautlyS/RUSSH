<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { useP2P } from '@/composables/useP2P';
import { 
  Plus, Search, Star, Folder, Server, Users, 
  ChevronDown, ChevronRight, LayoutDashboard, 
  Settings, Zap, FolderOpen, Circle
} from 'lucide-vue-next';
import ConnectionItem from '@/components/connections/ConnectionItem.vue';

defineProps<{
  collapsed: boolean;
}>();

const router = useRouter();
const route = useRoute();
const connectionStore = useConnectionStore();
const { peers, isOnline } = useP2P();

const searchQuery = ref('');
const expandedFolders = ref<Set<string>>(new Set());

const favoriteProfiles = computed(() => 
  connectionStore.profiles.filter(p => p.tags.includes('favorite'))
);

const folders = computed(() => connectionStore.folders);

const ungroupedProfiles = computed(() => 
  connectionStore.profiles.filter(p => !p.folder && !p.tags.includes('favorite'))
);

const navItems = [
  { path: '/dashboard', icon: LayoutDashboard, label: 'DASH' },
  { path: '/connections', icon: Server, label: 'CONNECT' },
  { path: '/p2p', icon: Users, label: 'P2P' },
  { path: '/settings', icon: Settings, label: 'CONFIG' },
];

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

function isActive(path: string) {
  return route.path === path || route.path.startsWith(path + '/');
}

function getPeerId(peer: { peerId?: string }): string {
  return peer.peerId || 'unknown';
}
</script>

<template>
  <aside :class="['sidebar', collapsed && 'sidebar--collapsed']">
    <!-- New Button -->
    <div class="sidebar-top">
      <button @click="router.push('/connections/new')" class="sidebar-new-btn">
        <Plus class="w-4 h-4" />
        <span v-if="!collapsed" class="text-[8px]">NEW</span>
      </button>
    </div>
    
    <!-- Search -->
    <div v-if="!collapsed" class="sidebar-search">
      <Search class="sidebar-search-icon" />
      <input v-model="searchQuery" type="text" placeholder="FIND..." class="sidebar-search-input" />
    </div>
    
    <!-- Nav -->
    <nav class="sidebar-nav">
      <router-link 
        v-for="item in navItems" 
        :key="item.path"
        :to="item.path"
        :class="['sidebar-nav-item', isActive(item.path) && 'sidebar-nav-item--active']"
      >
        <component :is="item.icon" class="w-4 h-4 flex-shrink-0" />
        <span v-if="!collapsed">{{ item.label }}</span>
      </router-link>
    </nav>
    
    <div class="sidebar-divider" />
    
    <!-- Connections -->
    <div class="sidebar-content">
      <!-- Favorites -->
      <div v-if="favoriteProfiles.length > 0" class="sidebar-group">
        <div v-if="!collapsed" class="sidebar-group-title sidebar-group-title--yellow">
          <Star class="w-3 h-3" />
          FAVS
        </div>
        <div class="sidebar-group-items">
          <ConnectionItem 
            v-for="profile in favoriteProfiles" 
            :key="profile.id"
            :profile="profile"
            :collapsed="collapsed"
          />
        </div>
      </div>
      
      <!-- Folders -->
      <div v-for="folder in folders" :key="folder" class="sidebar-group">
        <button 
          v-if="!collapsed"
          @click="toggleFolder(folder)"
          class="sidebar-folder-btn"
        >
          <component :is="isFolderExpanded(folder) ? ChevronDown : ChevronRight" class="w-3 h-3" />
          <component :is="isFolderExpanded(folder) ? FolderOpen : Folder" class="w-3 h-3" />
          {{ folder }}
        </button>
        <div v-if="isFolderExpanded(folder) || collapsed" class="sidebar-group-items">
          <ConnectionItem 
            v-for="profile in getProfilesByFolder(folder)" 
            :key="profile.id"
            :profile="profile"
            :collapsed="collapsed"
          />
        </div>
      </div>
      
      <!-- Ungrouped -->
      <div v-if="ungroupedProfiles.length > 0" class="sidebar-group">
        <div v-if="!collapsed" class="sidebar-group-title">
          <Server class="w-3 h-3" />
          SERVERS
        </div>
        <div class="sidebar-group-items">
          <ConnectionItem 
            v-for="profile in ungroupedProfiles" 
            :key="profile.id"
            :profile="profile"
            :collapsed="collapsed"
          />
        </div>
      </div>
      
      <!-- Peers -->
      <div v-if="peers.length > 0" class="sidebar-group">
        <div v-if="!collapsed" class="sidebar-group-title sidebar-group-title--purple">
          <Zap class="w-3 h-3" />
          PEERS
        </div>
        <div class="sidebar-group-items">
          <div v-for="peer in peers" :key="getPeerId(peer)" class="sidebar-peer">
            <Circle class="w-2 h-2 sidebar-peer-dot" />
            <span v-if="!collapsed" class="sidebar-peer-id">{{ getPeerId(peer).slice(0, 6) }}</span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Footer -->
    <div class="sidebar-footer">
      <div :class="['sidebar-status', isOnline && 'sidebar-status--online']">
        <div :class="['sidebar-status-dot', isOnline ? 'sidebar-status-dot--online' : 'sidebar-status-dot--offline']" />
        <span v-if="!collapsed">{{ isOnline ? 'ONLINE' : 'OFFLINE' }}</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  width: 200px;
  background: var(--pixel-dark);
  border-right: 3px solid var(--pixel-border);
  transition: width 150ms;
}

.sidebar--collapsed {
  width: 60px;
}

.sidebar-top {
  padding: 8px;
}

.sidebar-new-btn {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 38px;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  background: var(--pixel-green-dark);
  border: 3px solid var(--pixel-green);
  color: var(--pixel-black);
  font-weight: bold;
  box-shadow: 2px 2px 0 var(--pixel-black);
}

.sidebar-new-btn:active {
  transform: translate(1px, 1px);
  box-shadow: 1px 1px 0 var(--pixel-black);
}

.sidebar-search {
  padding: 0 8px 8px;
  position: relative;
}

.sidebar-search-icon {
  position: absolute;
  left: 16px;
  top: 50%;
  transform: translateY(-50%);
  width: 12px;
  height: 12px;
  color: var(--pixel-muted);
}

.sidebar-search-input {
  width: 100%;
  height: 32px;
  padding: 0 8px 0 28px;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  background: var(--pixel-black);
  border: 2px solid var(--pixel-border);
  color: var(--pixel-white);
}

.sidebar-search-input:focus {
  border-color: var(--pixel-green);
  outline: none;
}

.sidebar-search-input::placeholder {
  color: var(--pixel-muted);
}

.sidebar-nav {
  padding: 0 4px 8px;
}

.sidebar-nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  margin-bottom: 4px;
  font-family: 'Press Start 2P', monospace;
  font-size: 9px;
  color: var(--pixel-text);
  background: transparent;
  border: 2px solid transparent;
  transition: all 100ms;
}

.sidebar-nav-item:hover {
  color: var(--pixel-green);
  background: var(--pixel-mid);
  border-color: var(--pixel-border);
}

.sidebar-nav-item--active {
  color: var(--pixel-green);
  background: var(--pixel-dark);
  border-color: var(--pixel-green);
  box-shadow: inset -2px -2px 0 var(--pixel-mid), 0 0 10px rgba(0, 255, 136, 0.2);
}

.sidebar-divider {
  height: 3px;
  margin: 0 8px;
  background: repeating-linear-gradient(90deg, var(--pixel-border) 0px, var(--pixel-border) 6px, transparent 6px, transparent 12px);
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 4px;
  scrollbar-width: none;
}

.sidebar-content::-webkit-scrollbar {
  display: none;
}

.sidebar-group {
  margin-bottom: 12px;
}

.sidebar-group-title {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  font-family: 'Press Start 2P', monospace;
  font-size: 7px;
  color: var(--pixel-text);
  text-transform: uppercase;
}

.sidebar-group-title--yellow {
  color: var(--pixel-yellow);
}

.sidebar-group-title--purple {
  color: var(--pixel-purple);
}

.sidebar-folder-btn {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  font-family: 'Press Start 2P', monospace;
  font-size: 7px;
  color: var(--pixel-blue);
  text-transform: uppercase;
  text-align: left;
}

.sidebar-folder-btn:hover {
  color: var(--pixel-cyan);
}

.sidebar-group-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 4px;
}

.sidebar-peer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border: 2px solid var(--pixel-border);
}

.sidebar-peer-dot {
  color: var(--pixel-green);
  fill: var(--pixel-green);
}

.sidebar-peer-id {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--pixel-white);
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar-footer {
  padding: 8px;
  border-top: 3px solid var(--pixel-border);
}

.sidebar-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  background: var(--pixel-mid);
  border: 2px solid var(--pixel-border);
  color: var(--pixel-text);
}

.sidebar-status--online {
  background: var(--pixel-green-dark);
  border-color: var(--pixel-green);
  color: var(--pixel-black);
}

.sidebar-status-dot {
  width: 8px;
  height: 8px;
  border: 2px solid;
}

.sidebar-status-dot--online {
  background: var(--pixel-green);
  border-color: var(--pixel-black);
  box-shadow: 0 0 6px var(--pixel-green);
  animation: pulse 1.5s infinite;
}

.sidebar-status-dot--offline {
  background: var(--pixel-muted);
  border-color: var(--pixel-border);
}

@keyframes pulse {
  0%, 100% { box-shadow: 0 0 6px var(--pixel-green); }
  50% { box-shadow: 0 0 12px var(--pixel-green), 0 0 20px var(--pixel-green); }
}
</style>
