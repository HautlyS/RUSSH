<script setup lang="ts">
defineProps<{
  size?: 'sm' | 'md' | 'lg';
  color?: string;
}>();
</script>

<template>
  <div 
    :class="[
      'animate-spin rounded-full border-2 border-current border-t-transparent',
      {
        'w-4 h-4': size === 'sm',
        'w-6 h-6': size === 'md' || !size,
        'w-8 h-8': size === 'lg',
      }
    ]"
    :style="color ? { color } : {}"
    role="status"
    aria-label="Loading"
  >
    <span class="sr-only">Loading...</span>
  </div>
</template>
