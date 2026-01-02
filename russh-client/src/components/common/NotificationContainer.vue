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

function getColorClass(type: Notification['type']) {
  switch (type) {
    case 'success': return 'bg-green-50 dark:bg-green-900/30 border-green-200 dark:border-green-800 text-green-800 dark:text-green-200';
    case 'error': return 'bg-red-50 dark:bg-red-900/30 border-red-200 dark:border-red-800 text-red-800 dark:text-red-200';
    case 'warning': return 'bg-yellow-50 dark:bg-yellow-900/30 border-yellow-200 dark:border-yellow-800 text-yellow-800 dark:text-yellow-200';
    default: return 'bg-blue-50 dark:bg-blue-900/30 border-blue-200 dark:border-blue-800 text-blue-800 dark:text-blue-200';
  }
}

function getIconColorClass(type: Notification['type']) {
  switch (type) {
    case 'success': return 'text-green-500';
    case 'error': return 'text-red-500';
    case 'warning': return 'text-yellow-500';
    default: return 'text-blue-500';
  }
}

function dismiss(id: string) {
  notificationStore.markAsRead(id);
}
</script>

<template>
  <div class="fixed bottom-4 right-4 z-50 space-y-2 max-w-sm">
    <TransitionGroup name="notification">
      <div 
        v-for="notification in visibleNotifications" 
        :key="notification.id"
        :class="[
          'flex items-start gap-3 p-4 rounded-lg border shadow-lg',
          getColorClass(notification.type)
        ]"
      >
        <component 
          :is="getIcon(notification.type)" 
          :class="['w-5 h-5 flex-shrink-0', getIconColorClass(notification.type)]" 
        />
        
        <div class="flex-1 min-w-0">
          <div class="font-medium">{{ notification.title }}</div>
          <div class="text-sm opacity-80">{{ notification.message }}</div>
          
          <button 
            v-if="notification.action"
            @click="notification.action.handler(); dismiss(notification.id)"
            class="mt-2 text-sm font-medium underline hover:no-underline"
          >
            {{ notification.action.label }}
          </button>
        </div>
        
        <button 
          @click="dismiss(notification.id)"
          class="p-1 hover:bg-black/10 dark:hover:bg-white/10 rounded"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.notification-enter-active,
.notification-leave-active {
  transition: all 0.3s ease;
}

.notification-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.notification-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

.notification-move {
  transition: transform 0.3s ease;
}
</style>
