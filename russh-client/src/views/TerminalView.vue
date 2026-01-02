<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import TerminalContainer from '@/components/terminal/TerminalContainer.vue';

const route = useRoute();
const sessionId = computed(() => route.params.sessionId as string);
</script>

<template>
  <div class="h-full">
    <TerminalContainer :session-id="sessionId" />
  </div>
</template>
