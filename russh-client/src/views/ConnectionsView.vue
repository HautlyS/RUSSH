<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { useVisualEffects } from '@/composables/useVisualEffects';
import { Plus, Search, Grid, List } from 'lucide-vue-next';
import ConnectionCard from '@/components/connections/ConnectionCard.vue';
import ConnectionList from '@/components/connections/ConnectionList.vue';
import QuickConnect from '@/components/connections/QuickConnect.vue';
import GradualBlur from '@/components/extra/GradualBlur.vue';

import type { ConnectionProfile } from '@/types/ssh';

const router = useRouter();
const connectionStore = useConnectionStore();
const { isGradualBlurEnabled, visualEffects } = useVisualEffects();

const searchQuery = ref('');
const viewMode = ref<'grid' | 'list'>('list');
const showQuickConnect = ref(false);

const filteredProfiles = computed(() => {
  if (!searchQuery.value) return connectionStore.sortedProfiles;
  const query = searchQuery.value.toLowerCase();
  return connectionStore.sortedProfiles.filter((p: ConnectionProfile) => 
    p.name.toLowerCase().includes(query) ||
    p.host.toLowerCase().includes(query) ||
    p.username.toLowerCase().includes(query)
  );
});

function handleConnect(profile: ConnectionProfile) {
  connectionStore.connect(profile.id);
}

function handleEdit(profile: ConnectionProfile) {
  router.push(`/connections/${profile.id}/edit`);
}

async function handleDelete(profile: ConnectionProfile) {
  if (confirm(`Delete "${profile.name}"?`)) {
    await connectionStore.deleteProfile(profile.id);
  }
}

function handleQuickConnected(sessionId: string) {
  showQuickConnect.value = false;
  router.push(`/terminal/${sessionId}`);
}
</script>

<template>
  <div class="p-6">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Connections</h1>
      <div class="flex gap-2">
        <button 
          @click="showQuickConnect = !showQuickConnect" 
          class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg"
        >
          Quick Connect
        </button>
        <button @click="router.push('/connections/new')" class="btn-primary">
          <Plus class="w-4 h-4" />
          New Connection
        </button>
      </div>
    </div>
    
    <!-- Quick Connect Panel -->
    <div v-if="showQuickConnect" class="mb-6">
      <QuickConnect @connected="handleQuickConnected" @cancel="showQuickConnect = false" />
    </div>
    
    <!-- Toolbar -->
    <div class="flex items-center gap-4 mb-6">
      <div class="relative flex-1 max-w-md">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input 
          v-model="searchQuery"
          type="text"
          placeholder="Search connections..."
          class="w-full pl-10 pr-4 py-2 bg-gray-50 dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>
      
      <div class="flex items-center gap-1 bg-gray-100 dark:bg-gray-800 rounded-lg p-1">
        <button 
          @click="viewMode = 'list'"
          class="p-2 rounded"
          :class="viewMode === 'list' ? 'bg-white dark:bg-gray-700 shadow-sm' : ''"
        >
          <List class="w-4 h-4" />
        </button>
        <button 
          @click="viewMode = 'grid'"
          class="p-2 rounded"
          :class="viewMode === 'grid' ? 'bg-white dark:bg-gray-700 shadow-sm' : ''"
        >
          <Grid class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- List View -->
    <div v-if="viewMode === 'list'" class="relative">
      <GradualBlur
        v-if="isGradualBlurEnabled"
        position="top"
        :height="visualEffects.gradualBlur.height"
        :strength="visualEffects.gradualBlur.strength"
        curve="bezier"
        target="parent"
        class="z-10"
      />
      <ConnectionList 
        :filter="searchQuery"
        :show-folders="true"
        @connect="handleConnect"
        @edit="handleEdit"
        @delete="handleDelete"
      />
      <GradualBlur
        v-if="isGradualBlurEnabled"
        position="bottom"
        :height="visualEffects.gradualBlur.height"
        :strength="visualEffects.gradualBlur.strength"
        curve="bezier"
        target="parent"
        class="z-10"
      />
    </div>
    
    <!-- Grid View -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <ConnectionCard 
        v-for="profile in filteredProfiles" 
        :key="profile.id"
        :profile="profile"
        @connect="handleConnect(profile)"
        @edit="handleEdit(profile)"
        @delete="handleDelete(profile)"
      />
    </div>
    
    <!-- Empty State -->
    <div v-if="filteredProfiles.length === 0" class="text-center py-12">
      <p class="text-gray-500 dark:text-gray-400 mb-4">
        {{ searchQuery ? 'No connections match your search' : 'No connections yet' }}
      </p>
      <button v-if="!searchQuery" @click="router.push('/connections/new')" class="btn-primary">
        <Plus class="w-4 h-4" />
        New Connection
      </button>
    </div>
  </div>
</template>
