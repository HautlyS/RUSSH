<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import FileBrowser from '@/components/files/FileBrowser.vue';

const route = useRoute();
const sessionId = computed(() => route.params.sessionId as string);
</script>

<template>
  <div class="h-full">
    <FileBrowser v-if="sessionId" :session-id="sessionId" />
    <div v-else class="h-full flex items-center justify-center text-gray-500 dark:text-gray-400">
      <div class="text-center">
        <p class="mb-2">No active session</p>
        <p class="text-sm">Connect to a server to browse files</p>
      </div>
    </div>
  </div>
</template>
