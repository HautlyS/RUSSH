<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
}>();
</script>

<template>
  <button
    :class="[
      'pixel-btn',
      variant === 'primary' && 'pixel-btn-primary',
      variant === 'danger' && 'pixel-btn-danger',
      variant === 'ghost' && 'pixel-btn-ghost',
      size === 'sm' && 'pixel-btn-sm',
      size === 'lg' && 'px-6 py-3',
      loading && 'animate-pixel-pulse',
      disabled && 'opacity-50 cursor-not-allowed'
    ]"
    :disabled="disabled || loading"
  >
    <span v-if="loading" class="animate-pixel-blink">...</span>
    <slot v-else />
  </button>
</template>
