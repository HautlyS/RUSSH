<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { useVisualEffects } from '@/composables/useVisualEffects';
import { Plus, Clock, Star, Server, ArrowRight } from 'lucide-vue-next';
import DecryptedText from '@/components/extra/DecryptedText.vue';
import RotatingText from '@/components/extra/RotatingText.vue';
import ElectricBorder from '@/components/extra/ElectricBorder.vue';

const router = useRouter();
const connectionStore = useConnectionStore();
const { 
  isDecryptedTextEnabled, 
  isRotatingTextEnabled, 
  isElectricBorderEnabled,
  visualEffects 
} = useVisualEffects();

const recentProfiles = computed(() => connectionStore.recentProfiles);
const favoriteProfiles = computed(() => connectionStore.favoriteProfiles);
const connectedCount = computed(() => connectionStore.connectedProfiles.length);
const totalProfiles = computed(() => connectionStore.profiles.length);

const rotatingMessages = [
  'Secure connections',
  'Cross-device sync', 
  'P2P networking',
  'Fast transfers'
];
</script>

<template>
  <div class="p-6 max-w-6xl mx-auto">
    <!-- Header -->
    <div class="mb-8">
      <h1 class="text-2xl font-bold mb-2">
        <DecryptedText
          v-if="isDecryptedTextEnabled"
          text="Welcome to RUSSH"
          :speed="visualEffects.decryptedText.speed"
          :max-iterations="visualEffects.decryptedText.maxIterations"
          :sequential="visualEffects.decryptedText.sequential"
          animate-on="view"
          reveal-direction="start"
          class-name="text-gray-900 dark:text-white"
          encrypted-class-name="text-green-500"
        />
        <span v-else>Welcome to RUSSH</span>
      </h1>
      <p class="text-gray-500 flex items-center gap-2">
        <span v-if="!isRotatingTextEnabled">Cross-device SSH client for secure remote connections</span>
        <RotatingText
          v-else
          :texts="rotatingMessages"
          :rotation-interval="visualEffects.rotatingText.rotationInterval"
          :stagger-duration="visualEffects.rotatingText.staggerDuration"
          main-class-name="text-gray-500"
          split-level-class-name="overflow-hidden"
          :transition="{ type: 'spring', damping: 30, stiffness: 400 }"
        />
      </p>
    </div>
    
    <!-- Quick Stats -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
      <div class="card p-4">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center">
            <Server class="w-5 h-5 text-green-600 dark:text-green-400" />
          </div>
          <div>
            <div class="text-2xl font-bold">{{ connectedCount }}</div>
            <div class="text-sm text-gray-500">Active Connections</div>
          </div>
        </div>
      </div>
      
      <div class="card p-4">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
            <Star class="w-5 h-5 text-blue-600 dark:text-blue-400" />
          </div>
          <div>
            <div class="text-2xl font-bold">{{ favoriteProfiles.length }}</div>
            <div class="text-sm text-gray-500">Favorites</div>
          </div>
        </div>
      </div>
      
      <div class="card p-4">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 bg-purple-100 dark:bg-purple-900/30 rounded-lg flex items-center justify-center">
            <Server class="w-5 h-5 text-purple-600 dark:text-purple-400" />
          </div>
          <div>
            <div class="text-2xl font-bold">{{ totalProfiles }}</div>
            <div class="text-sm text-gray-500">Total Profiles</div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Quick Actions -->
    <div class="mb-8">
      <h2 class="text-lg font-semibold mb-4">Quick Actions</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <ElectricBorder
          v-if="isElectricBorderEnabled"
          :color="visualEffects.electricBorder.color"
          :speed="visualEffects.electricBorder.speed"
          :chaos="visualEffects.electricBorder.chaos"
          :thickness="visualEffects.electricBorder.thickness"
          :style="{ borderRadius: '12px' }"
        >
          <button 
            @click="router.push('/connections/new')"
            class="w-full card p-4 flex items-center gap-4 hover:border-blue-500 transition-colors text-left"
          >
            <div class="w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
              <Plus class="w-6 h-6 text-blue-600 dark:text-blue-400" />
            </div>
            <div class="flex-1">
              <div class="font-medium">New Connection</div>
              <div class="text-sm text-gray-500">Create a new SSH connection profile</div>
            </div>
            <ArrowRight class="w-5 h-5 text-gray-400" />
          </button>
        </ElectricBorder>
        <button 
          v-else
          @click="router.push('/connections/new')"
          class="card p-4 flex items-center gap-4 hover:border-blue-500 transition-colors text-left"
        >
          <div class="w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
            <Plus class="w-6 h-6 text-blue-600 dark:text-blue-400" />
          </div>
          <div class="flex-1">
            <div class="font-medium">New Connection</div>
            <div class="text-sm text-gray-500">Create a new SSH connection profile</div>
          </div>
          <ArrowRight class="w-5 h-5 text-gray-400" />
        </button>
        
        <ElectricBorder
          v-if="isElectricBorderEnabled"
          :color="'#a855f7'"
          :speed="visualEffects.electricBorder.speed"
          :chaos="visualEffects.electricBorder.chaos"
          :thickness="visualEffects.electricBorder.thickness"
          :style="{ borderRadius: '12px' }"
        >
          <button 
            @click="router.push('/p2p')"
            class="w-full card p-4 flex items-center gap-4 hover:border-blue-500 transition-colors text-left"
          >
            <div class="w-12 h-12 bg-purple-100 dark:bg-purple-900/30 rounded-lg flex items-center justify-center">
              <Server class="w-6 h-6 text-purple-600 dark:text-purple-400" />
            </div>
            <div class="flex-1">
              <div class="font-medium">P2P Connections</div>
              <div class="text-sm text-gray-500">Connect to devices behind NAT</div>
            </div>
            <ArrowRight class="w-5 h-5 text-gray-400" />
          </button>
        </ElectricBorder>
        <button 
          v-else
          @click="router.push('/p2p')"
          class="card p-4 flex items-center gap-4 hover:border-blue-500 transition-colors text-left"
        >
          <div class="w-12 h-12 bg-purple-100 dark:bg-purple-900/30 rounded-lg flex items-center justify-center">
            <Server class="w-6 h-6 text-purple-600 dark:text-purple-400" />
          </div>
          <div class="flex-1">
            <div class="font-medium">P2P Connections</div>
            <div class="text-sm text-gray-500">Connect to devices behind NAT</div>
          </div>
          <ArrowRight class="w-5 h-5 text-gray-400" />
        </button>
      </div>
    </div>
    
    <!-- Recent Connections -->
    <div v-if="recentProfiles.length > 0" class="mb-8">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold flex items-center gap-2">
          <Clock class="w-5 h-5" />
          Recent Connections
        </h2>
        <router-link to="/connections" class="text-sm text-blue-500 hover:underline">
          View all
        </router-link>
      </div>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <button 
          v-for="profile in recentProfiles" 
          :key="profile.id"
          @click="connectionStore.connect(profile.id)"
          class="card p-4 text-left hover:border-blue-500 transition-colors"
        >
          <div class="flex items-center gap-3">
            <div 
              class="w-2 h-8 rounded-full"
              :style="{ backgroundColor: profile.color || '#6b7280' }"
            />
            <div class="flex-1 min-w-0">
              <div class="font-medium truncate">{{ profile.name }}</div>
              <div class="text-sm text-gray-500 truncate">{{ profile.username }}@{{ profile.host }}</div>
            </div>
          </div>
        </button>
      </div>
    </div>
    
    <!-- Empty State -->
    <div v-if="totalProfiles === 0" class="text-center py-12">
      <Server class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
      <h3 class="text-lg font-medium mb-2">No connections yet</h3>
      <p class="text-gray-500 mb-4">Create your first SSH connection to get started</p>
      <button @click="router.push('/connections/new')" class="btn-primary">
        <Plus class="w-4 h-4" />
        New Connection
      </button>
    </div>
  </div>
</template>
