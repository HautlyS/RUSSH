<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';

export interface ContextMenuItem {
  label?: string;
  icon?: any;
  shortcut?: string;
  action?: () => void;
  type?: 'separator';
  disabled?: boolean;
}

const props = defineProps<{
  x: number;
  y: number;
  items: ContextMenuItem[];
}>();

const emit = defineEmits<{
  close: [];
}>();

const menuRef = ref<HTMLElement | null>(null);
const adjustedX = ref(props.x);
const adjustedY = ref(props.y);

function handleClickOutside(event: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    emit('close');
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    emit('close');
  }
}

function executeAction(item: ContextMenuItem) {
  if (item.disabled) return;
  item.action?.();
  emit('close');
}

// Adjust position to stay within viewport
watch([() => props.x, () => props.y], () => {
  if (!menuRef.value) return;
  
  const rect = menuRef.value.getBoundingClientRect();
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;
  
  adjustedX.value = props.x;
  adjustedY.value = props.y;
  
  if (props.x + rect.width > viewportWidth) {
    adjustedX.value = viewportWidth - rect.width - 8;
  }
  
  if (props.y + rect.height > viewportHeight) {
    adjustedY.value = viewportHeight - rect.height - 8;
  }
}, { immediate: true });

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  document.removeEventListener('keydown', handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div 
      ref="menuRef"
      class="fixed z-50 min-w-[160px] py-1 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700"
      :style="{ left: `${adjustedX}px`, top: `${adjustedY}px` }"
    >
      <template v-for="(item, index) in items" :key="index">
        <div 
          v-if="item.type === 'separator'"
          class="my-1 border-t border-gray-200 dark:border-gray-700"
        />
        <button 
          v-else
          @click="executeAction(item)"
          :disabled="item.disabled"
          :class="[
            'w-full flex items-center gap-3 px-3 py-1.5 text-sm text-left',
            item.disabled 
              ? 'text-gray-400 cursor-not-allowed' 
              : 'hover:bg-gray-100 dark:hover:bg-gray-700'
          ]"
        >
          <component v-if="item.icon" :is="item.icon" class="w-4 h-4" />
          <span class="flex-1">{{ item.label }}</span>
          <span v-if="item.shortcut" class="text-xs text-gray-400">{{ item.shortcut }}</span>
        </button>
      </template>
    </div>
  </Teleport>
</template>
