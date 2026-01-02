/**
 * Notifications store - manages in-app notifications
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  timestamp: Date;
  read: boolean;
  action?: {
    label: string;
    handler: () => void;
  };
  autoClose?: boolean;
  duration?: number;
}

export const useNotificationStore = defineStore('notifications', () => {
  // State
  const notifications = ref<Notification[]>([]);
  const maxNotifications = 100;

  // Getters
  const unreadCount = computed(() => 
    notifications.value.filter(n => !n.read).length
  );

  const recentNotifications = computed(() => 
    notifications.value.slice(0, 10)
  );

  const unreadNotifications = computed(() => 
    notifications.value.filter(n => !n.read)
  );

  // Actions
  function addNotification(
    type: Notification['type'],
    title: string,
    message: string,
    options?: {
      action?: Notification['action'];
      autoClose?: boolean;
      duration?: number;
    }
  ): string {
    const notification: Notification = {
      id: crypto.randomUUID(),
      type,
      title,
      message,
      timestamp: new Date(),
      read: false,
      action: options?.action,
      autoClose: options?.autoClose ?? true,
      duration: options?.duration ?? 5000,
    };
    
    notifications.value.unshift(notification);
    
    // Keep only last N notifications
    if (notifications.value.length > maxNotifications) {
      notifications.value = notifications.value.slice(0, maxNotifications);
    }
    
    // Auto-close if enabled
    if (notification.autoClose) {
      setTimeout(() => {
        markAsRead(notification.id);
      }, notification.duration);
    }
    
    return notification.id;
  }

  function markAsRead(id: string) {
    const notification = notifications.value.find(n => n.id === id);
    if (notification && !notification.read) {
      notification.read = true;
    }
  }

  function markAllAsRead() {
    notifications.value.forEach(n => n.read = true);
  }

  function removeNotification(id: string) {
    const index = notifications.value.findIndex(n => n.id === id);
    if (index !== -1) {
      notifications.value.splice(index, 1);
    }
  }

  function clearAll() {
    notifications.value = [];
  }

  function clearRead() {
    notifications.value = notifications.value.filter(n => !n.read);
  }

  // Convenience methods
  function success(title: string, message: string, action?: Notification['action']) {
    return addNotification('success', title, message, { action });
  }

  function error(title: string, message: string, action?: Notification['action']) {
    return addNotification('error', title, message, { action, autoClose: false });
  }

  function warning(title: string, message: string, action?: Notification['action']) {
    return addNotification('warning', title, message, { action });
  }

  function info(title: string, message: string, action?: Notification['action']) {
    return addNotification('info', title, message, { action });
  }

  return {
    // State
    notifications,
    // Getters
    unreadCount,
    recentNotifications,
    unreadNotifications,
    // Actions
    addNotification,
    markAsRead,
    markAllAsRead,
    removeNotification,
    clearAll,
    clearRead,
    // Convenience
    success,
    error,
    warning,
    info,
  };
});
