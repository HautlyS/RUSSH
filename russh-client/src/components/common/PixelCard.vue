<script setup lang="ts">
defineProps<{
  glow?: boolean;
  hover?: boolean;
}>();
</script>

<template>
  <div
    :class="[
      glow ? 'pixel-card-glow' : 'pixel-card',
      hover && 'cursor-pointer'
    ]"
  >
    <slot />
  </div>
</template>
