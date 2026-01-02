<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted } from 'vue';
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
import ClickSpark from '@/components/extra/ClickSpark.vue';
import Noise from '@/components/extra/Noise.vue';

const settingsStore = useSettingsStore();
const connectionStore = useConnectionStore();
// Initialize theme (applies dark mode class to document)
useTheme();
const { isMobile } = usePlatform();
const { 
  isClickSparkEnabled, 
  isNoiseEnabled, 
  visualEffects,
  accentColor 
} = useVisualEffects();

// Initialize keyboard shortcuts
useKeyboard();

const sidebarCollapsed = ref(false);

// Computed spark color based on theme
const sparkColor = computed(() => {
  return visualEffects.value.clickSpark.color || accentColor.value || '#27FF64';
});

// Toggle sidebar
function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
}

// Listen for sidebar toggle events
onMounted(async () => {
  document.addEventListener('toggle-sidebar', toggleSidebar);
  
  // Load initial data
  await settingsStore.loadSettings();
  await connectionStore.loadProfiles();
});

// Clean up event listener
onUnmounted(() => {
  document.removeEventListener('toggle-sidebar', toggleSidebar);
});
</script>

<template>
  <ClickSpark
    v-if="isClickSparkEnabled"
    :spark-color="sparkColor"
    :spark-count="visualEffects.clickSpark.sparkCount"
    :spark-size="visualEffects.clickSpark.sparkSize"
    :spark-radius="visualEffects.clickSpark.sparkRadius"
    :duration="visualEffects.clickSpark.duration"
    easing="ease-out"
    class="h-screen"
  >
    <div class="h-full flex flex-col bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
      <!-- Noise Overlay -->
      <Noise
        v-if="isNoiseEnabled"
        :pattern-alpha="visualEffects.noise.alpha"
        :pattern-refresh-interval="visualEffects.noise.refreshInterval"
        :mix-blend-mode="visualEffects.noise.mixBlendMode"
        class="fixed inset-0 pointer-events-none z-[1000]"
      />

      <!-- Desktop Layout -->
      <template v-if="!isMobile">
        <AppHeader @toggle-sidebar="toggleSidebar" />
        
        <div class="flex-1 flex overflow-hidden">
          <AppSidebar :collapsed="sidebarCollapsed" />
          
          <main class="flex-1 overflow-auto">
            <router-view />
          </main>
        </div>
      </template>
      
      <!-- Mobile Layout -->
      <template v-else>
        <main class="flex-1 overflow-auto pb-16">
          <router-view />
        </main>
        
        <MobileNavigation />
      </template>
      
      <!-- Global Components -->
      <CommandPalette />
      <NotificationContainer />
    </div>
  </ClickSpark>

  <!-- Fallback when ClickSpark is disabled -->
  <div v-if="!isClickSparkEnabled" class="h-screen flex flex-col bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
    <!-- Noise Overlay -->
    <Noise
      v-if="isNoiseEnabled"
      :pattern-alpha="visualEffects.noise.alpha"
      :pattern-refresh-interval="visualEffects.noise.refreshInterval"
      :mix-blend-mode="visualEffects.noise.mixBlendMode"
      class="fixed inset-0 pointer-events-none z-[1000]"
    />

    <!-- Desktop Layout -->
    <template v-if="!isMobile">
      <AppHeader @toggle-sidebar="toggleSidebar" />
      
      <div class="flex-1 flex overflow-hidden">
        <AppSidebar :collapsed="sidebarCollapsed" />
        
        <main class="flex-1 overflow-auto">
          <router-view />
        </main>
      </div>
    </template>
    
    <!-- Mobile Layout -->
    <template v-else>
      <main class="flex-1 overflow-auto pb-16">
        <router-view />
      </main>
      
      <MobileNavigation />
    </template>
    
    <!-- Global Components -->
    <CommandPalette />
    <NotificationContainer />
  </div>
</template>
