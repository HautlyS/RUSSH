<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

defineProps<{
  mood?: 'happy' | 'thinking' | 'working' | 'sleeping';
  size?: number;
}>();

const frame = ref(0);
let interval: number;

// Simple idle animation
onMounted(() => {
  interval = setInterval(() => {
    frame.value = (frame.value + 1) % 4;
  }, 500);
});

onUnmounted(() => clearInterval(interval));
</script>

<template>
  <div 
    class="inline-block"
    :style="{ 
      width: (size || 32) + 'px', 
      height: (size || 32) + 'px',
      imageRendering: 'pixelated'
    }"
  >
    <!-- Pixel Robot Mascot -->
    <svg 
      :viewBox="'0 0 16 16'" 
      class="w-full h-full"
      style="image-rendering: pixelated"
    >
      <!-- Body -->
      <rect x="4" y="8" width="8" height="6" fill="var(--pixel-mid)" />
      <rect x="5" y="9" width="6" height="4" fill="var(--pixel-dark)" />
      
      <!-- Head -->
      <rect x="3" y="2" width="10" height="6" fill="var(--pixel-green)" />
      <rect x="4" y="3" width="8" height="4" fill="var(--pixel-green-dark)" />
      
      <!-- Eyes -->
      <rect 
        :x="frame % 2 === 0 ? 5 : 6" 
        y="4" 
        width="2" 
        height="2" 
        fill="var(--pixel-black)" 
      />
      <rect 
        :x="frame % 2 === 0 ? 9 : 10" 
        y="4" 
        width="2" 
        height="2" 
        fill="var(--pixel-black)" 
      />
      
      <!-- Eye shine -->
      <rect x="5" y="4" width="1" height="1" fill="var(--pixel-white)" />
      <rect x="9" y="4" width="1" height="1" fill="var(--pixel-white)" />
      
      <!-- Antenna -->
      <rect x="7" y="0" width="2" height="2" fill="var(--pixel-cyan)" />
      <rect 
        x="7" 
        :y="frame === 1 || frame === 3 ? -1 : 0" 
        width="2" 
        height="1" 
        :fill="frame === 1 || frame === 3 ? 'var(--pixel-yellow)' : 'var(--pixel-cyan)'" 
      />
      
      <!-- Arms -->
      <rect x="2" :y="10 + (frame % 2)" width="2" height="3" fill="var(--pixel-light)" />
      <rect x="12" :y="10 + ((frame + 1) % 2)" width="2" height="3" fill="var(--pixel-light)" />
      
      <!-- Legs -->
      <rect x="5" y="14" width="2" height="2" fill="var(--pixel-light)" />
      <rect x="9" y="14" width="2" height="2" fill="var(--pixel-light)" />
      
      <!-- Mouth based on mood -->
      <template v-if="mood === 'happy' || !mood">
        <rect x="6" y="6" width="4" height="1" fill="var(--pixel-black)" />
        <rect x="5" y="5" width="1" height="1" fill="var(--pixel-black)" />
        <rect x="10" y="5" width="1" height="1" fill="var(--pixel-black)" />
      </template>
      <template v-else-if="mood === 'thinking'">
        <rect x="6" y="6" width="3" height="1" fill="var(--pixel-black)" />
      </template>
      <template v-else-if="mood === 'working'">
        <rect x="6" y="5" width="4" height="2" fill="var(--pixel-black)" />
      </template>
      <template v-else-if="mood === 'sleeping'">
        <rect x="5" y="4" width="2" height="1" fill="var(--pixel-black)" />
        <rect x="9" y="4" width="2" height="1" fill="var(--pixel-black)" />
        <rect x="7" y="6" width="2" height="1" fill="var(--pixel-black)" />
      </template>
    </svg>
  </div>
</template>
