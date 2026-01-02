/**
 * Notification composable - handles in-app and system notifications
 */

import { useNotificationStore } from '@/stores/notifications';
import { useSettingsStore } from '@/stores/settings';
import { 
  isPermissionGranted, 
  requestPermission, 
  sendNotification 
} from '@tauri-apps/plugin-notification';

export function useNotification() {
  const notificationStore = useNotificationStore();
  const settingsStore = useSettingsStore();

  async function notify(
    type: 'info' | 'success' | 'warning' | 'error',
    title: string,
    message: string,
    options?: { 
      action?: { label: string; handler: () => void }; 
      system?: boolean;
      sound?: boolean;
    }
  ) {
    const settings = settingsStore.settings.notifications;
    
    if (!settings.enabled) return;

    // Add to in-app notifications
    notificationStore.addNotification(type, title, message, {
      action: options?.action,
      autoClose: type !== 'error',
    });

    // Play sound if enabled
    if (options?.sound && settings.sound) {
      playNotificationSound();
    }

    // Send system notification if requested and app is not focused
    if (options?.system && !document.hasFocus()) {
      await sendSystemNotification(title, message);
    }
  }

  async function sendSystemNotification(title: string, body: string) {
    try {
      let permissionGranted = await isPermissionGranted();
      
      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
      }
      
      if (permissionGranted) {
        sendNotification({ title, body });
      }
    } catch (e) {
      console.error('Failed to send system notification:', e);
    }
  }

  function playNotificationSound() {
    // Create a simple beep sound
    try {
      const audioContext = new AudioContext();
      const oscillator = audioContext.createOscillator();
      const gainNode = audioContext.createGain();
      
      oscillator.connect(gainNode);
      gainNode.connect(audioContext.destination);
      
      oscillator.frequency.value = 800;
      oscillator.type = 'sine';
      
      gainNode.gain.setValueAtTime(0.1, audioContext.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.1);
      
      oscillator.start(audioContext.currentTime);
      oscillator.stop(audioContext.currentTime + 0.1);
    } catch (e) {
      // Audio not available
    }
  }

  // Convenience methods
  function success(title: string, message: string, action?: { label: string; handler: () => void }) {
    return notify('success', title, message, { action });
  }

  function error(title: string, message: string, action?: { label: string; handler: () => void }) {
    return notify('error', title, message, { action, system: true });
  }

  function info(title: string, message: string, action?: { label: string; handler: () => void }) {
    return notify('info', title, message, { action });
  }

  function warning(title: string, message: string, action?: { label: string; handler: () => void }) {
    return notify('warning', title, message, { action });
  }

  // Connection-specific notifications
  function connectionEstablished(host: string) {
    const settings = settingsStore.settings.notifications;
    if (settings.connectionEvents) {
      success('Connected', `Successfully connected to ${host}`);
    }
  }

  function connectionLost(host: string) {
    const settings = settingsStore.settings.notifications;
    if (settings.connectionEvents) {
      error('Connection Lost', `Lost connection to ${host}`, {
        label: 'Reconnect',
        handler: () => {
          // Trigger reconnect
          document.dispatchEvent(new CustomEvent('reconnect-request', { detail: { host } }));
        }
      });
    }
  }

  function transferComplete(filename: string, type: 'upload' | 'download') {
    const settings = settingsStore.settings.notifications;
    if (settings.transferComplete) {
      success(
        type === 'upload' ? 'Upload Complete' : 'Download Complete',
        `${filename} transferred successfully`
      );
    }
  }

  return {
    notify,
    success,
    error,
    info,
    warning,
    connectionEstablished,
    connectionLost,
    transferComplete,
  };
}
