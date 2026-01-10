<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { usePlatform } from '@/composables/usePlatform';
import { Plus, Search, Grid, List, Server, Star, X, Zap } from 'lucide-vue-next';
import ConnectionCard from '@/components/connections/ConnectionCard.vue';
import ConnectionList from '@/components/connections/ConnectionList.vue';
import QuickConnect from '@/components/connections/QuickConnect.vue';
import PixelMascot from '@/components/extra/PixelMascot.vue';
import type { ConnectionProfile } from '@/types/ssh';

const router = useRouter();
const connectionStore = useConnectionStore();
const { isMobile, hapticFeedback } = usePlatform();

const searchQuery = ref('');
const viewMode = ref<'grid' | 'list'>('grid');
const showQuickConnect = ref(false);
const activeFilter = ref<'all' | 'favorites' | 'connected'>('all');

const filteredProfiles = computed(() => {
  let profiles = connectionStore.sortedProfiles;
  
  if (activeFilter.value === 'favorites') {
    profiles = profiles.filter((p: ConnectionProfile) => p.tags?.includes('favorite'));
  } else if (activeFilter.value === 'connected') {
    profiles = profiles.filter((p: ConnectionProfile) => 
      connectionStore.activeConnections.has(p.id)
    );
  }
  
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    profiles = profiles.filter((p: ConnectionProfile) => 
      p.name.toLowerCase().includes(query) ||
      p.host.toLowerCase().includes(query) ||
      p.username.toLowerCase().includes(query) ||
      p.tags?.some(t => t.toLowerCase().includes(query))
    );
  }
  
  return profiles;
});

const filterCounts = computed(() => ({
  all: connectionStore.profiles.length,
  favorites: connectionStore.profiles.filter(p => p.tags?.includes('favorite')).length,
  connected: connectionStore.connectedProfiles.length,
}));

function handleConnect(profile: ConnectionProfile) {
  hapticFeedback('medium');
  connectionStore.connect(profile.id);
}

function handleEdit(profile: ConnectionProfile) {
  hapticFeedback('light');
  router.push(`/connections/${profile.id}/edit`);
}

async function handleDelete(profile: ConnectionProfile) {
  hapticFeedback('warning');
  if (confirm(`Delete "${profile.name}"?`)) {
    await connectionStore.deleteProfile(profile.id);
  }
}

function handleQuickConnected(sessionId: string) {
  showQuickConnect.value = false;
  router.push(`/terminal/${sessionId}`);
}

function setFilter(filter: 'all' | 'favorites' | 'connected') {
  hapticFeedback('light');
  activeFilter.value = filter;
}
</script>

<template>
  <div class="connections-container">
    <!-- Header -->
    <header class="connections-header">
      <div>
        <h1 class="text-[14px] sm:text-[12px] pixel-glow-cyan" style="color: var(--pixel-cyan)">
          SESSIONS
        </h1>
        <p class="text-[9px] sm:text-[8px]" style="color: var(--pixel-light)">
          {{ filterCounts.all }} HOSTS · {{ filterCounts.connected }} ACTIVE
        </p>
      </div>
      
      <div class="flex items-center gap-2">
        <button 
          @click="showQuickConnect = !showQuickConnect" 
          class="connections-btn"
          :class="{ 'connections-btn--active': showQuickConnect }"
        >
          <Zap class="w-4 h-4 sm:w-3 sm:h-3" />
          <span class="hidden sm:inline">QUICK</span>
        </button>
        <button 
          @click="router.push('/connections/new')" 
          class="connections-btn connections-btn--primary"
        >
          <Plus class="w-4 h-4 sm:w-3 sm:h-3" />
          <span class="hidden sm:inline">NEW</span>
        </button>
      </div>
    </header>
    
    <!-- Quick Connect Panel -->
    <div v-if="showQuickConnect" class="connections-quick-panel">
      <QuickConnect 
        @connected="handleQuickConnected" 
        @cancel="showQuickConnect = false" 
      />
    </div>
    
    <!-- Toolbar -->
    <div class="connections-toolbar">
      <!-- Filters - Scrollable on mobile -->
      <div class="connections-filters">
        <button 
          v-for="filter in [
            { key: 'all', icon: Server, label: 'ALL' },
            { key: 'favorites', icon: Star, label: 'FAVS' },
          ]" 
          :key="filter.key"
          @click="setFilter(filter.key as 'all' | 'favorites')"
          class="connections-filter-btn"
          :class="{ 'connections-filter-btn--active': activeFilter === filter.key }"
        >
          <component :is="filter.icon" class="w-3.5 h-3.5 sm:w-3 sm:h-3" />
          <span>{{ filter.label }}</span>
          <span class="connections-filter-count">{{ filterCounts[filter.key as keyof typeof filterCounts] }}</span>
        </button>
        <button 
          @click="setFilter('connected')"
          class="connections-filter-btn"
          :class="{ 'connections-filter-btn--connected': activeFilter === 'connected' }"
        >
          <span class="connections-live-dot" />
          <span>LIVE</span>
          <span class="connections-filter-count">{{ filterCounts.connected }}</span>
        </button>
      </div>
      
      <!-- Search & View Toggle -->
      <div class="connections-search-row">
        <div class="connections-search">
          <Search class="connections-search-icon" />
          <input 
            v-model="searchQuery"
            type="text"
            placeholder="SEARCH..."
            class="connections-search-input"
          />
          <button 
            v-if="searchQuery"
            @click="searchQuery = ''"
            class="connections-search-clear"
          >
            <X class="w-3 h-3" />
          </button>
        </div>
        
        <div class="connections-view-toggle">
          <button 
            @click="viewMode = 'grid'"
            class="connections-view-btn"
            :class="{ 'connections-view-btn--active': viewMode === 'grid' }"
          >
            <Grid class="w-4 h-4 sm:w-3 sm:h-3" />
          </button>
          <button 
            @click="viewMode = 'list'"
            class="connections-view-btn"
            :class="{ 'connections-view-btn--active': viewMode === 'list' }"
          >
            <List class="w-4 h-4 sm:w-3 sm:h-3" />
          </button>
        </div>
      </div>
    </div>

    <!-- Grid View -->
    <div 
      v-if="viewMode === 'grid' && filteredProfiles.length > 0" 
      class="connections-grid"
    >
      <ConnectionCard 
        v-for="profile in filteredProfiles" 
        :key="profile.id"
        :profile="profile"
        @connect="handleConnect(profile)"
        @edit="handleEdit(profile)"
        @delete="handleDelete(profile)"
      />
    </div>
    
    <!-- List View -->
    <div v-else-if="viewMode === 'list' && filteredProfiles.length > 0">
      <ConnectionList 
        :filter="searchQuery"
        :show-folders="true"
        @connect="handleConnect"
        @edit="handleEdit"
        @delete="handleDelete"
      />
    </div>
    
    <!-- Empty State -->
    <div v-if="filteredProfiles.length === 0" class="connections-empty">
      <PixelMascot 
        :mood="searchQuery ? 'thinking' : 'sleeping'" 
        :size="isMobile ? 56 : 48" 
        class="mb-4"
      />
      <h3 class="text-[12px] sm:text-[11px] mb-2" style="color: var(--pixel-white)">
        {{ searchQuery ? 'NO MATCHES' : activeFilter !== 'all' ? `NO ${activeFilter.toUpperCase()}` : 'NO SESSIONS' }}
      </h3>
      <p class="text-[9px] sm:text-[8px] mb-4" style="color: var(--pixel-light)">
        {{ searchQuery ? 'TRY DIFFERENT SEARCH' : 'ADD YOUR FIRST SSH CONNECTION' }}
      </p>
      <div class="flex justify-center gap-2">
        <button 
          v-if="searchQuery || activeFilter !== 'all'" 
          @click="searchQuery = ''; activeFilter = 'all'" 
          class="connections-btn"
        >
          CLEAR
        </button>
        <button 
          v-if="!searchQuery" 
          @click="router.push('/connections/new')" 
          class="connections-btn connections-btn--primary"
        >
          <Plus class="w-3 h-3" />
          NEW
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.connections-container {
  padding: 16px;
  max-width: 1200px;
  margin: 0 auto;
}

@media (min-width: 640px) {
  .connections-container {
    padding: 24px;
  }
}

.connections-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.connections-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 40px;
  padding: 0 14px;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  background: var(--pixel-mid);
  border: 2px solid var(--pixel-light);
  color: var(--pixel-white);
  -webkit-tap-highlight-color: transparent;
}

@media (min-width: 640px) {
  .connections-btn {
    height: 32px;
    padding: 0 12px;
  }
}

.connections-btn:active {
  transform: translate(1px, 1px);
}

.connections-btn--primary {
  background: var(--pixel-green-dark);
  border-color: var(--pixel-green);
  color: var(--pixel-black);
}

.connections-btn--active {
  background: var(--pixel-cyan);
  border-color: var(--pixel-cyan);
  color: var(--pixel-black);
}

.connections-quick-panel {
  padding: 16px;
  margin-bottom: 16px;
  background: var(--pixel-dark);
  border: 3px solid var(--pixel-light);
}

.connections-toolbar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
}

@media (min-width: 640px) {
  .connections-toolbar {
    flex-direction: row;
    align-items: center;
    gap: 16px;
  }
}

.connections-filters {
  display: flex;
  gap: 4px;
  padding: 4px;
  background: var(--pixel-dark);
  border: 2px solid var(--pixel-mid);
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}

.connections-filters::-webkit-scrollbar {
  display: none;
}

.connections-filter-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 12px;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  white-space: nowrap;
  background: transparent;
  border: 2px solid transparent;
  color: var(--pixel-light);
  -webkit-tap-highlight-color: transparent;
}

@media (min-width: 640px) {
  .connections-filter-btn {
    padding: 6px 10px;
  }
}

.connections-filter-btn--active {
  background: var(--pixel-mid);
  border-color: var(--pixel-light);
  color: var(--pixel-white);
}

.connections-filter-btn--connected {
  background: rgba(0, 255, 136, 0.15);
  border-color: var(--pixel-green);
  color: var(--pixel-green);
}

.connections-filter-count {
  color: var(--pixel-light);
}

.connections-live-dot {
  width: 8px;
  height: 8px;
  background: var(--pixel-green);
  border: 1px solid var(--pixel-black);
  flex-shrink: 0;
}

.connections-search-row {
  display: flex;
  gap: 8px;
  flex: 1;
}

.connections-search {
  position: relative;
  flex: 1;
  max-width: 100%;
}

@media (min-width: 640px) {
  .connections-search {
    max-width: 280px;
  }
}

.connections-search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  width: 14px;
  height: 14px;
  color: var(--pixel-light);
  pointer-events: none;
}

.connections-search-input {
  width: 100%;
  height: 40px;
  padding: 0 36px;
  font-family: 'Press Start 2P', monospace;
  font-size: 9px;
  background: var(--pixel-black);
  border: 2px solid var(--pixel-light);
  color: var(--pixel-white);
}

@media (min-width: 640px) {
  .connections-search-input {
    height: 32px;
    font-size: 8px;
  }
}

.connections-search-input:focus {
  border-color: var(--pixel-green);
  outline: none;
}

.connections-search-input::placeholder {
  color: var(--pixel-light);
}

.connections-search-clear {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  padding: 4px;
  color: var(--pixel-light);
}

.connections-view-toggle {
  display: flex;
  gap: 2px;
  padding: 2px;
  background: var(--pixel-dark);
  border: 2px solid var(--pixel-mid);
}

.connections-view-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: transparent;
  color: var(--pixel-light);
  -webkit-tap-highlight-color: transparent;
}

@media (min-width: 640px) {
  .connections-view-btn {
    width: 28px;
    height: 28px;
  }
}

.connections-view-btn--active {
  background: var(--pixel-mid);
  color: var(--pixel-white);
}

.connections-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
}

@media (min-width: 480px) {
  .connections-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (min-width: 900px) {
  .connections-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

.connections-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 16px;
  text-align: center;
}
</style>
