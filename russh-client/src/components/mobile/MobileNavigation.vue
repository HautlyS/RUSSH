<script setup lang="ts">
import { computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { LayoutDashboard, Server, Users, Settings, Plus } from 'lucide-vue-next';
import { useConnectionStore } from '@/stores/connections';
import { usePlatform } from '@/composables/usePlatform';

const router = useRouter();
const route = useRoute();
const connectionStore = useConnectionStore();
const { hapticFeedback } = usePlatform();

const connectedCount = computed(() => connectionStore.connectedProfiles.length);

const navItems = [
  { path: '/dashboard', icon: LayoutDashboard, label: 'HOME' },
  { path: '/connections', icon: Server, label: 'SSH' },
  { path: '/p2p', icon: Users, label: 'P2P' },
  { path: '/settings', icon: Settings, label: 'CFG' },
];

function isActive(path: string): boolean {
  if (path === '/dashboard') return route.path === '/dashboard' || route.path === '/';
  return route.path.startsWith(path);
}

function navigate(path: string) {
  hapticFeedback('light');
  router.push(path);
}

function handleNewConnection() {
  hapticFeedback('medium');
  router.push('/connections/new');
}
</script>

<template>
  <nav 
    class="fixed bottom-0 left-0 right-0 z-50"
    style="
      background: var(--pixel-dark);
      border-top: 3px solid var(--pixel-light);
      padding-bottom: env(safe-area-inset-bottom, 0px);
    "
  >
    <!-- Rainbow pixel line -->
    <div 
      class="absolute top-0 left-0 right-0 h-[3px]"
      style="background: linear-gradient(90deg, var(--pixel-green), var(--pixel-cyan), var(--pixel-pink), var(--pixel-purple), var(--pixel-yellow))"
    />
    
    <div class="flex items-center justify-around h-16 px-2">
      <!-- Nav Items - Left side -->
      <button
        v-for="(item, index) in navItems.slice(0, 2)"
        :key="item.path"
        @click="navigate(item.path)"
        class="mobile-nav-btn"
        :class="{ 'mobile-nav-btn--active': isActive(item.path) }"
      >
        <div class="mobile-nav-icon" :class="{ 'mobile-nav-icon--active': isActive(item.path) }">
          <component :is="item.icon" class="w-5 h-5" />
          <span 
            v-if="item.path === '/connections' && connectedCount > 0"
            class="mobile-nav-badge"
          >
            {{ connectedCount > 9 ? '9+' : connectedCount }}
          </span>
        </div>
        <span class="mobile-nav-label">{{ item.label }}</span>
      </button>
      
      <!-- Center FAB -->
      <div class="relative flex items-center justify-center w-16">
        <button
          @click="handleNewConnection"
          class="mobile-fab"
          aria-label="New connection"
        >
          <Plus class="w-6 h-6" style="color: var(--pixel-black)" />
        </button>
      </div>
      
      <!-- Nav Items - Right side -->
      <button
        v-for="item in navItems.slice(2)"
        :key="item.path"
        @click="navigate(item.path)"
        class="mobile-nav-btn"
        :class="{ 'mobile-nav-btn--active': isActive(item.path) }"
      >
        <div class="mobile-nav-icon" :class="{ 'mobile-nav-icon--active': isActive(item.path) }">
          <component :is="item.icon" class="w-5 h-5" />
        </div>
        <span class="mobile-nav-label">{{ item.label }}</span>
      </button>
    </div>
  </nav>
</template>

<style scoped>
.mobile-nav-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  height: 100%;
  min-width: 56px;
  padding: 8px 4px;
  color: var(--pixel-light);
  -webkit-tap-highlight-color: transparent;
  touch-action: manipulation;
}

.mobile-nav-btn--active {
  color: var(--pixel-green);
}

.mobile-nav-icon {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 36px;
  border: 2px solid transparent;
  transition: all 100ms;
}

.mobile-nav-icon--active {
  background: rgba(0, 255, 136, 0.15);
  border-color: var(--pixel-green);
  box-shadow: 0 0 12px rgba(0, 255, 136, 0.3);
}

.mobile-nav-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 8px;
  font-family: 'Press Start 2P', monospace;
  background: var(--pixel-green);
  border: 2px solid var(--pixel-black);
  color: var(--pixel-black);
}

.mobile-nav-label {
  font-size: 8px;
  font-family: 'Press Start 2P', monospace;
  margin-top: 4px;
}

.mobile-fab {
  position: absolute;
  top: -20px;
  width: 52px;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--pixel-green);
  border: 3px solid var(--pixel-green-dark);
  box-shadow: 
    4px 4px 0 var(--pixel-black),
    0 0 20px rgba(0, 255, 136, 0.5);
  -webkit-tap-highlight-color: transparent;
  touch-action: manipulation;
}

.mobile-fab:active {
  transform: translate(2px, 2px);
  box-shadow: 
    2px 2px 0 var(--pixel-black),
    0 0 15px rgba(0, 255, 136, 0.4);
}
</style>
