<script setup lang="ts">
import { computed } from 'vue';
import { useNotificationStore, type Notification } from '@/stores/notifications';
import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from 'lucide-vue-next';

const notificationStore = useNotificationStore();

const visibleNotifications = computed(() => 
  notificationStore.notifications.filter(n => !n.read).slice(0, 5)
);

function getIcon(type: Notification['type']) {
  switch (type) {
    case 'success': return CheckCircle;
    case 'error': return AlertCircle;
    case 'warning': return AlertTriangle;
    default: return Info;
  }
}

function getPixelStyle(type: Notification['type']) {
  switch (type) {
    case 'success': return { bg: 'var(--pixel-green)', border: 'var(--pixel-green-dark)', text: 'var(--pixel-black)' };
    case 'error': return { bg: 'var(--pixel-red)', border: '#cc3a47', text: 'var(--pixel-white)' };
    case 'warning': return { bg: 'var(--pixel-yellow)', border: 'var(--pixel-orange)', text: 'var(--pixel-black)' };
    default: return { bg: 'var(--pixel-blue)', border: 'var(--pixel-cyan)', text: 'var(--pixel-black)' };
  }
}

function dismiss(id: string) {
  notificationStore.markAsRead(id);
}
</script>

<template>
  <div class="fixed bottom-4 right-4 z-50 space-y-2 max-w-xs">
    <TransitionGroup name="notification">
      <div 
        v-for="notification in visibleNotifications" 
        :key="notification.id"
        class="flex items-start gap-2 p-3 animate-pixel-pop"
        :style="{
          background: getPixelStyle(notification.type).bg,
          border: '3px solid ' + getPixelStyle(notification.type).border,
          boxShadow: '4px 4px 0 var(--pixel-black)',
          color: getPixelStyle(notification.type).text
        }"
      >
        <component 
          :is="getIcon(notification.type)" 
          class="w-4 h-4 flex-shrink-0 mt-0.5"
        />
        
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold uppercase">{{ notification.title }}</div>
          <div class="text-[8px] opacity-90">{{ notification.message }}</div>
          
          <button 
            v-if="notification.action"
            @click="notification.action.handler(); dismiss(notification.id)"
            class="mt-1 text-[8px] underline hover:no-underline"
          >
            {{ notification.action.label }}
          </button>
        </div>
        
        <button 
          @click="dismiss(notification.id)"
          class="p-1 hover:opacity-70"
        >
          <X class="w-3 h-3" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.notification-enter-active,
.notification-leave-active {
  transition: all 0.2s steps(4);
}

.notification-enter-from {
  opacity: 0;
  transform: translateX(100%) scale(0.8);
}

.notification-leave-to {
  opacity: 0;
  transform: translateX(100%) scale(0.8);
}

.notification-move {
  transition: transform 0.2s steps(4);
}
</style>
