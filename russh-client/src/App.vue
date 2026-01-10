<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted } from 'vue';
import { useRoute } from 'vue-router';
import { useSettingsStore } from '@/stores/settings';
import { useConnectionStore } from '@/stores/connections';
import { useTheme } from '@/composables/useTheme';
import { useKeyboard } from '@/composables/useKeyboard';
import { usePlatform } from '@/composables/usePlatform';
import { useVisualEffects } from '@/composables/useVisualEffects';
import AppHeader from '@/components/common/AppHeader.vue';
import AppSidebar from '@/components/common/AppSidebar.vue';
import CommandPalette from '@/components/common/CommandPalette.vue';
import NotificationContainer from '@/components/common/NotificationContainer.vue';
import MobileNavigation from '@/components/mobile/MobileNavigation.vue';
import VoxelParticles from '@/components/extra/VoxelParticles.vue';

const route = useRoute();
const settingsStore = useSettingsStore();
const connectionStore = useConnectionStore();
useTheme();
const { isMobile } = usePlatform();
const { isNoiseEnabled } = useVisualEffects();
useKeyboard();

const sidebarCollapsed = ref(false);
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
const isLandingPage = computed(() => route.name === 'home' && !isTauri);

const pixelColors = ['#00ff88', '#00ffff', '#ff6b9d', '#b967ff', '#fffb00'];

function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
}

onMounted(async () => {
  document.addEventListener('toggle-sidebar', toggleSidebar);
  await settingsStore.loadSettings();
  await connectionStore.loadProfiles();
});

onUnmounted(() => {
  document.removeEventListener('toggle-sidebar', toggleSidebar);
});
</script>

<template>
  <!-- Landing Page -->
  <router-view v-if="isLandingPage" />

  <!-- App Layout -->
  <div v-else class="h-screen flex flex-col overflow-hidden" style="background: var(--pixel-black)">
    <!-- Pixel Grid Background -->
    <div class="fixed inset-0 pointer-events-none pixel-grid-bg opacity-30" />
    
    <!-- Voxel Particles -->
    <VoxelParticles
      :colors="pixelColors"
      :count="25"
      :speed="0.3"
      :size="5"
      :gravity="0.015"
      class="fixed inset-0 z-0 opacity-60"
    />
    
    <!-- Scanline Effect -->
    <div
      v-if="isNoiseEnabled"
      class="fixed inset-0 pointer-events-none z-[100] opacity-5"
      style="background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.3) 2px, rgba(0,0,0,0.3) 4px)"
    />

    <!-- Desktop Layout -->
    <template v-if="!isMobile">
      <AppHeader @toggle-sidebar="toggleSidebar" />
      <div class="flex-1 flex overflow-hidden relative z-10">
        <AppSidebar :collapsed="sidebarCollapsed" />
        <main class="flex-1 overflow-auto">
          <router-view />
        </main>
      </div>
    </template>
    
    <!-- Mobile Layout -->
    <template v-else>
      <main class="flex-1 overflow-y-auto overflow-x-hidden pb-20 relative z-10 touch-pan-y overscroll-contain">
        <router-view />
      </main>
      <MobileNavigation />
    </template>
    
    <CommandPalette />
    <NotificationContainer />
  </div>
</template>
