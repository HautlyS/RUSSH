<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';
import { Home, Server, Terminal, FolderOpen, Settings } from 'lucide-vue-next';

const router = useRouter();
const route = useRoute();

const navItems = [
  { path: '/', icon: Home, label: 'Home' },
  { path: '/connections', icon: Server, label: 'Servers' },
  { path: '/terminal', icon: Terminal, label: 'Terminal' },
  { path: '/files', icon: FolderOpen, label: 'Files' },
  { path: '/settings', icon: Settings, label: 'Settings' },
];

function isActive(path: string): boolean {
  if (path === '/') return route.path === '/';
  return route.path.startsWith(path);
}

function navigate(path: string) {
  router.push(path);
}
</script>

<template>
  <nav class="mobile-navigation fixed bottom-0 left-0 right-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 safe-area-bottom">
    <div class="flex items-center justify-around h-16">
      <button
        v-for="item in navItems"
        :key="item.path"
        @click="navigate(item.path)"
        class="flex flex-col items-center justify-center flex-1 h-full transition-colors"
        :class="isActive(item.path) 
          ? 'text-blue-600 dark:text-blue-400' 
          : 'text-gray-500 dark:text-gray-400'"
      >
        <component :is="item.icon" class="w-6 h-6" />
        <span class="text-xs mt-1">{{ item.label }}</span>
      </button>
    </div>
  </nav>
</template>

<style scoped>
.safe-area-bottom {
  padding-bottom: env(safe-area-inset-bottom, 0);
}
</style>
