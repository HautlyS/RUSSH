<script setup lang="ts">
import { computed } from 'vue';
import { Server, Play, Edit, Trash2, MoreVertical } from 'lucide-vue-next';
import { useConnectionStore } from '@/stores/connections';
import { useVisualEffects } from '@/composables/useVisualEffects';
import { usePlatform } from '@/composables/usePlatform';
import ConnectionStatus from './ConnectionStatus.vue';
import Magnet from '@/components/extra/Magnet.vue';
import DecryptedText from '@/components/extra/DecryptedText.vue';
import type { ConnectionProfile } from '@/types/ssh';

const props = defineProps<{
  profile: ConnectionProfile;
}>();

const emit = defineEmits<{
  (e: 'connect'): void;
  (e: 'edit'): void;
  (e: 'delete'): void;
}>();

const connectionStore = useConnectionStore();
const { isMagnetEnabled, isDecryptedTextEnabled, visualEffects } = useVisualEffects();
const { isMobile, isTouchDevice } = usePlatform();

// Disable magnet on mobile/touch devices
const shouldUseMagnet = computed(() => 
  isMagnetEnabled.value && !isMobile.value && !isTouchDevice()
);

const isConnected = computed(() => 
  connectionStore.activeConnections.has(props.profile.id)
);

const connectionState = computed(() => 
  connectionStore.connectionStates.get(props.profile.id)
);

// Accessibility labels
const cardLabel = computed(() => 
  `${props.profile.name} - ${props.profile.username}@${props.profile.host}:${props.profile.port}`
);

const statusLabel = computed(() => {
  const status = connectionState.value?.status;
  if (status === 'connected') return 'Connected';
  if (status === 'connecting') return 'Connecting';
  if (status === 'error') return 'Connection error';
  return 'Disconnected';
});
</script>

<template>
  <Magnet
    v-if="shouldUseMagnet"
    :padding="visualEffects.magnet.padding"
    :magnet-strength="visualEffects.magnet.magnetStrength"
    active-transition="transform 0.2s ease-out"
    inactive-transition="transform 0.4s ease-in-out"
  >
    <article 
      class="connection-card bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow"
      :aria-label="cardLabel"
      role="article"
    >
      <div class="flex items-start justify-between">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-blue-100 dark:bg-blue-900/30 rounded-lg" aria-hidden="true">
            <Server class="w-5 h-5 text-blue-600 dark:text-blue-400" />
          </div>
          <div>
            <h3 class="font-medium text-gray-900 dark:text-white">
              <DecryptedText
                v-if="isDecryptedTextEnabled"
                :text="profile.name"
                :speed="visualEffects.decryptedText.speed"
                animate-on="view"
                class-name="text-gray-900 dark:text-white"
                encrypted-class-name="text-green-500"
              />
              <span v-else>{{ profile.name }}</span>
            </h3>
            <p class="text-sm text-gray-500 dark:text-gray-400">
              {{ profile.username }}@{{ profile.host }}:{{ profile.port }}
            </p>
          </div>
        </div>
        
        <ConnectionStatus :state="connectionState?.status" :aria-label="statusLabel" />
      </div>
      
      <div v-if="profile.tags?.length" class="mt-3 flex flex-wrap gap-1" role="list" aria-label="Tags">
        <span 
          v-for="tag in profile.tags" 
          :key="tag"
          class="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded"
          role="listitem"
        >
          {{ tag }}
        </span>
      </div>
      
      <div class="mt-4 flex items-center gap-2" role="group" aria-label="Connection actions">
        <button
          v-if="!isConnected"
          @click="emit('connect')"
          class="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
          :aria-label="`Connect to ${profile.name}`"
        >
          <Play class="w-4 h-4" aria-hidden="true" />
          Connect
        </button>
        <button
          v-else
          @click="emit('connect')"
          class="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2"
          :aria-label="`Open terminal for ${profile.name}`"
        >
          Open Terminal
        </button>
        
        <button
          @click="emit('edit')"
          class="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500"
          :aria-label="`Edit ${profile.name}`"
        >
          <Edit class="w-4 h-4" aria-hidden="true" />
        </button>
        
        <button
          @click="emit('delete')"
          class="p-2 text-gray-500 hover:text-red-600 dark:text-gray-400 dark:hover:text-red-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-red-500"
          :aria-label="`Delete ${profile.name}`"
        >
          <Trash2 class="w-4 h-4" aria-hidden="true" />
        </button>
      </div>
    </article>
  </Magnet>

  <!-- Fallback without Magnet -->
  <article 
    v-else
    class="connection-card bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow"
    :aria-label="cardLabel"
    role="article"
  >
    <div class="flex items-start justify-between">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-blue-100 dark:bg-blue-900/30 rounded-lg" aria-hidden="true">
          <Server class="w-5 h-5 text-blue-600 dark:text-blue-400" />
        </div>
        <div>
          <h3 class="font-medium text-gray-900 dark:text-white">
            <DecryptedText
              v-if="isDecryptedTextEnabled"
              :text="profile.name"
              :speed="visualEffects.decryptedText.speed"
              animate-on="view"
              class-name="text-gray-900 dark:text-white"
              encrypted-class-name="text-green-500"
            />
            <span v-else>{{ profile.name }}</span>
          </h3>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            {{ profile.username }}@{{ profile.host }}:{{ profile.port }}
          </p>
        </div>
      </div>
      
      <ConnectionStatus :state="connectionState?.status" :aria-label="statusLabel" />
    </div>
    
    <div v-if="profile.tags?.length" class="mt-3 flex flex-wrap gap-1" role="list" aria-label="Tags">
      <span 
        v-for="tag in profile.tags" 
        :key="tag"
        class="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded"
        role="listitem"
      >
        {{ tag }}
      </span>
    </div>
    
    <div class="mt-4 flex items-center gap-2" role="group" aria-label="Connection actions">
      <button
        v-if="!isConnected"
        @click="emit('connect')"
        class="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
        :aria-label="`Connect to ${profile.name}`"
      >
        <Play class="w-4 h-4" aria-hidden="true" />
        Connect
      </button>
      <button
        v-else
        @click="emit('connect')"
        class="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2"
        :aria-label="`Open terminal for ${profile.name}`"
      >
        Open Terminal
      </button>
      
      <button
        @click="emit('edit')"
        class="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500"
        :aria-label="`Edit ${profile.name}`"
      >
        <Edit class="w-4 h-4" aria-hidden="true" />
      </button>
      
      <button
        @click="emit('delete')"
        class="p-2 text-gray-500 hover:text-red-600 dark:text-gray-400 dark:hover:text-red-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-red-500"
        :aria-label="`Delete ${profile.name}`"
      >
        <Trash2 class="w-4 h-4" aria-hidden="true" />
      </button>
    </div>
  </article>
</template>
