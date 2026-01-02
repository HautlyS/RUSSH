<script setup lang="ts">
import { computed } from 'vue';
import { useNotificationStore } from '@/stores/notifications';
import { useConnectionStore } from '@/stores/connections';
import { useVisualEffects } from '@/composables/useVisualEffects';
import { usePlatform } from '@/composables/usePlatform';
import { Menu, Search, Bell, Settings, Wifi, WifiOff } from 'lucide-vue-next';
import DecryptedText from '@/components/extra/DecryptedText.vue';
import Magnet from '@/components/extra/Magnet.vue';

defineEmits<{
  'toggle-sidebar': [];
}>();

const notificationStore = useNotificationStore();
const connectionStore = useConnectionStore();
const { isDecryptedTextEnabled, isMagnetEnabled } = useVisualEffects();
const { isMobile, isTouchDevice } = usePlatform();

const unreadCount = computed(() => notificationStore.unreadCount);
const connectedCount = computed(() => connectionStore.connectedProfiles.length);

// Disable magnet on mobile/touch
const shouldUseMagnet = computed(() => 
  isMagnetEnabled.value && !isMobile.value && !isTouchDevice()
);

function openCommandPalette() {
  document.dispatchEvent(new CustomEvent('open-command-palette'));
}

function toggleNotifications() {
  document.dispatchEvent(new CustomEvent('toggle-notifications'));
}
</script>

<template>
  <header class="h-12 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 flex items-center px-4 drag-region">
    <!-- Sidebar Toggle -->
    <button 
      @click="$emit('toggle-sidebar')" 
      class="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg no-drag transition-colors"
      aria-label="Toggle sidebar"
    >
      <Menu class="w-5 h-5" />
    </button>
    
    <!-- Logo with DecryptedText -->
    <div class="flex items-center gap-2 ml-2">
      <div class="w-6 h-6 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
        <span class="text-white text-xs font-bold">R</span>
      </div>
      <DecryptedText 
        v-if="isDecryptedTextEnabled"
        text="RUSSH"
        :speed="50"
        animate-on="view"
        class="font-semibold text-lg hidden sm:block"
      />
      <span v-else class="font-semibold text-lg hidden sm:block">RUSSH</span>
    </div>
    
    <!-- Search / Command Palette Trigger -->
    <div class="flex-1 max-w-md mx-4">
      <button 
        @click="openCommandPalette"
        class="w-full flex items-center gap-2 px-3 py-1.5 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg text-gray-500 text-sm no-drag transition-colors"
      >
        <Search class="w-4 h-4" />
        <span class="hidden sm:inline">Search or press</span>
        <kbd class="hidden sm:inline px-1.5 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs">⌘K</kbd>
      </button>
    </div>
    
    <!-- Right Actions -->
    <div class="flex items-center gap-1">
      <!-- Connection Status -->
      <div 
        class="flex items-center gap-1.5 px-2 py-1 rounded-lg text-sm"
        :class="connectedCount > 0 ? 'text-green-600 dark:text-green-400' : 'text-gray-400'"
      >
        <component :is="connectedCount > 0 ? Wifi : WifiOff" class="w-4 h-4" />
        <span class="hidden sm:inline">{{ connectedCount }}</span>
      </div>
      
      <!-- Notifications with Magnet -->
      <Magnet v-if="shouldUseMagnet" :padding="50" :magnet-strength="3">
        <button 
          @click="toggleNotifications" 
          class="relative p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg no-drag transition-colors"
          aria-label="Notifications"
        >
          <Bell class="w-5 h-5" />
          <span 
            v-if="unreadCount > 0" 
            class="absolute -top-0.5 -right-0.5 w-4 h-4 bg-red-500 text-white text-xs rounded-full flex items-center justify-center"
          >
            {{ unreadCount > 9 ? '9+' : unreadCount }}
          </span>
        </button>
      </Magnet>
      <button 
        v-else
        @click="toggleNotifications" 
        class="relative p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg no-drag transition-colors"
        aria-label="Notifications"
      >
        <Bell class="w-5 h-5" />
        <span 
          v-if="unreadCount > 0" 
          class="absolute -top-0.5 -right-0.5 w-4 h-4 bg-red-500 text-white text-xs rounded-full flex items-center justify-center"
        >
          {{ unreadCount > 9 ? '9+' : unreadCount }}
        </span>
      </button>
      
      <!-- Settings with Magnet -->
      <Magnet v-if="shouldUseMagnet" :padding="50" :magnet-strength="3">
        <router-link 
          to="/settings" 
          class="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg no-drag transition-colors"
          aria-label="Settings"
        >
          <Settings class="w-5 h-5" />
        </router-link>
      </Magnet>
      <router-link 
        v-else
        to="/settings" 
        class="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg no-drag transition-colors"
        aria-label="Settings"
      >
        <Settings class="w-5 h-5" />
      </router-link>
    </div>
  </header>
</template>
