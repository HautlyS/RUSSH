<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

defineProps<{
  size?: 'sm' | 'md' | 'lg';
  color?: string;
}>();

const frame = ref(0);
let interval: number;

// 8-bit loading animation frames
const frames = ['▓', '▒', '░', '▒'];

onMounted(() => {
  interval = setInterval(() => {
    frame.value = (frame.value + 1) % frames.length;
  }, 150);
});

onUnmounted(() => clearInterval(interval));
</script>

<template>
  <div 
    :class="[
      'inline-flex items-center justify-center pixel-text',
      size === 'sm' && 'text-[10px]',
      size === 'lg' && 'text-[16px]',
      (!size || size === 'md') && 'text-[12px]'
    ]"
    :style="{ color: color || 'var(--pixel-green)' }"
    role="status"
    aria-label="Loading"
  >
    <span class="animate-pixel-blink">{{ frames[frame] }}</span>
    <span class="animate-pixel-blink" style="animation-delay: 50ms">{{ frames[(frame + 1) % 4] }}</span>
    <span class="animate-pixel-blink" style="animation-delay: 100ms">{{ frames[(frame + 2) % 4] }}</span>
    <span class="sr-only">Loading...</span>
  </div>
</template>
