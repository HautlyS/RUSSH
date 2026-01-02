<script setup lang="ts">
import { computed } from 'vue';
import { Signal, SignalLow, SignalMedium, SignalHigh } from 'lucide-vue-next';

const props = defineProps<{
  latency?: number;
  connectionType?: 'direct' | 'relay';
}>();

const qualityLevel = computed(() => {
  if (!props.latency) return 'unknown';
  if (props.latency < 50) return 'excellent';
  if (props.latency < 150) return 'good';
  if (props.latency < 300) return 'fair';
  return 'poor';
});

const qualityConfig = computed(() => {
  switch (qualityLevel.value) {
    case 'excellent':
      return { icon: SignalHigh, color: 'text-green-500', label: 'Excellent' };
    case 'good':
      return { icon: SignalMedium, color: 'text-green-400', label: 'Good' };
    case 'fair':
      return { icon: SignalLow, color: 'text-yellow-500', label: 'Fair' };
    case 'poor':
      return { icon: Signal, color: 'text-red-500', label: 'Poor' };
    default:
      return { icon: Signal, color: 'text-gray-400', label: 'Unknown' };
  }
});
</script>

<template>
  <div class="connection-quality flex items-center gap-2" :title="`${qualityConfig.label}${latency ? ` (${latency}ms)` : ''}`">
    <component :is="qualityConfig.icon" class="w-4 h-4" :class="qualityConfig.color" />
    <div class="text-xs">
      <span v-if="latency" class="text-gray-600 dark:text-gray-400">{{ latency }}ms</span>
      <span v-if="connectionType" class="ml-1 px-1.5 py-0.5 rounded text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700">
        {{ connectionType }}
      </span>
    </div>
  </div>
</template>
