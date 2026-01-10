<script setup lang="ts">
import { computed } from 'vue';
import { useNotificationStore } from '@/stores/notifications';
import { useConnectionStore } from '@/stores/connections';
import { usePlatform } from '@/composables/usePlatform';
import { Menu, Search, Bell, Settings, Wifi, WifiOff, Terminal, Command } from 'lucide-vue-next';

defineEmits<{
  'toggle-sidebar': [];
}>();

const notificationStore = useNotificationStore();
const connectionStore = useConnectionStore();
const { isMobile } = usePlatform();

const unreadCount = computed(() => notificationStore.unreadCount);
const connectedCount = computed(() => connectionStore.connectedProfiles.length);

function openCommandPalette() {
  document.dispatchEvent(new CustomEvent('open-command-palette'));
}

function toggleNotifications() {
  document.dispatchEvent(new CustomEvent('toggle-notifications'));
}
</script>

<template>
  <header 
    class="h-[52px] flex items-center px-3 drag-region relative z-50"
    style="background: var(--pixel-dark); border-bottom: 3px solid var(--pixel-light)"
  >
    <!-- Pixel accent line -->
    <div 
      class="absolute top-0 left-0 right-0 h-[3px]"
      style="background: linear-gradient(90deg, var(--pixel-green), var(--pixel-cyan), var(--pixel-pink), var(--pixel-purple))"
    />
    
    <!-- Sidebar Toggle -->
    <button 
      @click="$emit('toggle-sidebar')" 
      class="pixel-btn-icon no-drag"
      aria-label="Toggle sidebar"
    >
      <Menu class="w-4 h-4" />
    </button>
    
    <!-- Logo -->
    <div class="flex items-center gap-2 ml-2">
      <div 
        class="w-8 h-8 flex items-center justify-center animate-pixel-float"
        style="background: var(--pixel-green); border: 3px solid var(--pixel-green-dark); box-shadow: 3px 3px 0 var(--pixel-black)"
      >
        <Terminal class="w-4 h-4" style="color: var(--pixel-black)" />
      </div>
      <span 
        class="hidden sm:block text-[12px] pixel-glow-green"
        style="color: var(--pixel-green)"
      >
        RUSSH
      </span>
    </div>
    
    <!-- Search / Command Palette -->
    <div class="flex-1 max-w-md mx-3">
      <button 
        @click="openCommandPalette"
        class="w-full flex items-center gap-2 px-3 py-1.5 pixel-border no-drag group text-[9px]"
        style="color: var(--pixel-light)"
      >
        <Search class="w-3 h-3 group-hover:text-[var(--pixel-green)]" />
        <span class="hidden sm:inline flex-1 text-left">SEARCH...</span>
        <div class="hidden sm:flex items-center gap-1">
          <kbd 
            class="px-1 py-0.5 text-[7px]"
            style="background: var(--pixel-mid); border: 2px solid var(--pixel-light)"
          >
            <Command class="w-2 h-2 inline" />
          </kbd>
          <kbd 
            class="px-1 py-0.5 text-[7px]"
            style="background: var(--pixel-mid); border: 2px solid var(--pixel-light)"
          >K</kbd>
        </div>
      </button>
    </div>
    
    <!-- Right Actions -->
    <div class="flex items-center gap-1">
      <!-- Connection Status -->
      <div 
        v-if="!isMobile"
        :class="[
          'flex items-center gap-1 px-2 py-1 mr-1 text-[8px]',
          connectedCount > 0 ? 'pixel-badge-success' : 'pixel-badge'
        ]"
      >
        <component :is="connectedCount > 0 ? Wifi : WifiOff" class="w-3 h-3" />
        <span>{{ connectedCount }}</span>
      </div>
      
      <!-- Notifications -->
      <button 
        @click="toggleNotifications" 
        class="pixel-btn-icon no-drag relative"
        aria-label="Notifications"
      >
        <Bell class="w-4 h-4" />
        <span 
          v-if="unreadCount > 0" 
          class="absolute -top-1 -right-1 min-w-[14px] h-[14px] px-0.5 flex items-center justify-center text-[7px]"
          style="background: var(--pixel-red); border: 2px solid var(--pixel-black); color: var(--pixel-white)"
        >
          {{ unreadCount > 9 ? '9+' : unreadCount }}
        </span>
      </button>
      
      <!-- Settings -->
      <router-link 
        to="/settings" 
        class="pixel-btn-icon no-drag"
        aria-label="Settings"
      >
        <Settings class="w-4 h-4" />
      </router-link>
    </div>
  </header>
</template>
