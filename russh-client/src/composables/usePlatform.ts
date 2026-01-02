/**
 * Platform composable - detects platform and provides platform-specific utilities
 */

import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export type Platform = 'windows' | 'macos' | 'linux' | 'ios' | 'android' | 'unknown';
export type HapticType = 'light' | 'medium' | 'heavy' | 'success' | 'warning' | 'error';

export function usePlatform() {
  const platform = ref<Platform>('unknown');
  const isTauri = ref(false);
  const isKeyboardVisible = ref(false);
  const keyboardHeight = ref(0);

  const isMobile = computed(() => 
    platform.value === 'ios' || platform.value === 'android'
  );

  const isDesktop = computed(() => 
    platform.value === 'windows' || platform.value === 'macos' || platform.value === 'linux'
  );

  const isMac = computed(() => platform.value === 'macos');
  const isWindows = computed(() => platform.value === 'windows');
  const isLinux = computed(() => platform.value === 'linux');
  const isIOS = computed(() => platform.value === 'ios');
  const isAndroid = computed(() => platform.value === 'android');

  async function detectPlatform() {
    // Check if running in Tauri
    isTauri.value = '__TAURI__' in window;

    if (isTauri.value) {
      try {
        const { platform: tauriPlatform } = await import('@tauri-apps/plugin-os');
        const os = await tauriPlatform();
        
        switch (os) {
          case 'darwin':
            platform.value = 'macos';
            break;
          case 'windows':
            platform.value = 'windows';
            break;
          case 'linux':
            platform.value = 'linux';
            break;
          case 'ios':
            platform.value = 'ios';
            break;
          case 'android':
            platform.value = 'android';
            break;
          default:
            platform.value = 'unknown';
        }
      } catch {
        detectFromUserAgent();
      }
    } else {
      detectFromUserAgent();
    }
  }

  function detectFromUserAgent() {
    const ua = navigator.userAgent.toLowerCase();
    
    if (/iphone|ipad|ipod/.test(ua)) {
      platform.value = 'ios';
    } else if (/android/.test(ua)) {
      platform.value = 'android';
    } else if (/mac/.test(ua)) {
      platform.value = 'macos';
    } else if (/win/.test(ua)) {
      platform.value = 'windows';
    } else if (/linux/.test(ua)) {
      platform.value = 'linux';
    } else {
      platform.value = 'unknown';
    }
  }

  // Haptic feedback
  async function hapticFeedback(type: HapticType = 'light') {
    if (!isMobile.value || !isTauri.value) return;
    
    try {
      await invoke('haptic_feedback', { type });
    } catch {
      // Fallback to vibration API
      if ('vibrate' in navigator) {
        const patterns: Record<HapticType, number[]> = {
          light: [10],
          medium: [20],
          heavy: [30],
          success: [10, 50, 10],
          warning: [20, 50, 20],
          error: [30, 50, 30, 50, 30],
        };
        navigator.vibrate(patterns[type]);
      }
    }
  }

  // Get modifier key name for current platform
  function getModifierKey(): string {
    return isMac.value ? '⌘' : 'Ctrl';
  }

  // Check if touch is supported
  function isTouchDevice(): boolean {
    return 'ontouchstart' in window || navigator.maxTouchPoints > 0;
  }

  // Get safe area insets (for mobile)
  function getSafeAreaInsets() {
    const style = getComputedStyle(document.documentElement);
    return {
      top: parseInt(style.getPropertyValue('env(safe-area-inset-top)') || '0'),
      right: parseInt(style.getPropertyValue('env(safe-area-inset-right)') || '0'),
      bottom: parseInt(style.getPropertyValue('env(safe-area-inset-bottom)') || '0'),
      left: parseInt(style.getPropertyValue('env(safe-area-inset-left)') || '0'),
    };
  }

  // Keyboard visibility handlers
  function onKeyboardShow(e: Event) {
    isKeyboardVisible.value = true;
    const event = e as CustomEvent;
    keyboardHeight.value = event.detail?.keyboardHeight || 300;
  }

  function onKeyboardHide() {
    isKeyboardVisible.value = false;
    keyboardHeight.value = 0;
  }

  // Visual viewport resize (for keyboard detection on web)
  function onViewportResize() {
    if (window.visualViewport) {
      const heightDiff = window.innerHeight - window.visualViewport.height;
      isKeyboardVisible.value = heightDiff > 150;
      keyboardHeight.value = heightDiff > 150 ? heightDiff : 0;
    }
  }

  onMounted(() => {
    detectPlatform();
    
    // Listen for keyboard events
    window.addEventListener('keyboardDidShow', onKeyboardShow);
    window.addEventListener('keyboardDidHide', onKeyboardHide);
    
    // Fallback for web
    if (window.visualViewport) {
      window.visualViewport.addEventListener('resize', onViewportResize);
    }
  });

  onUnmounted(() => {
    window.removeEventListener('keyboardDidShow', onKeyboardShow);
    window.removeEventListener('keyboardDidHide', onKeyboardHide);
    
    if (window.visualViewport) {
      window.visualViewport.removeEventListener('resize', onViewportResize);
    }
  });

  return {
    platform,
    isTauri,
    isMobile,
    isDesktop,
    isMac,
    isWindows,
    isLinux,
    isIOS,
    isAndroid,
    isKeyboardVisible,
    keyboardHeight,
    hapticFeedback,
    getModifierKey,
    isTouchDevice,
    getSafeAreaInsets,
  };
}
