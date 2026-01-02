/**
 * Platform composable - detects platform and provides platform-specific utilities
 */

import { ref, computed, onMounted, onUnmounted } from 'vue';

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
    isTauri.value = typeof window !== 'undefined' && '__TAURI__' in window;

    if (isTauri.value) {
      try {
        // Dynamic import for Tauri environment
        await import('@tauri-apps/api/window');
        detectFromUserAgent();
      } catch {
        detectFromUserAgent();
      }
    } else {
      detectFromUserAgent();
    }
  }

  function detectFromUserAgent() {
    const ua = navigator.userAgent.toLowerCase();
    
    if (/iphone|ipad|ipod/.test(ua)) platform.value = 'ios';
    else if (/android/.test(ua)) platform.value = 'android';
    else if (/mac/.test(ua)) platform.value = 'macos';
    else if (/win/.test(ua)) platform.value = 'windows';
    else if (/linux/.test(ua)) platform.value = 'linux';
    else platform.value = 'unknown';
  }

  async function hapticFeedback(type: HapticType = 'light') {
    if (!isMobile.value) return;
    
    if (isTauri.value) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('haptic_feedback', { type });
        return;
      } catch { /* fallback below */ }
    }
    
    if ('vibrate' in navigator) {
      const patterns: Record<HapticType, number[]> = {
        light: [10], medium: [20], heavy: [30],
        success: [10, 50, 10], warning: [20, 50, 20], error: [30, 50, 30, 50, 30],
      };
      navigator.vibrate(patterns[type]);
    }
  }

  function getModifierKey(): string {
    return isMac.value ? '⌘' : 'Ctrl';
  }

  function isTouchDevice(): boolean {
    return 'ontouchstart' in window || navigator.maxTouchPoints > 0;
  }

  function getSafeAreaInsets() {
    const style = getComputedStyle(document.documentElement);
    return {
      top: parseInt(style.getPropertyValue('env(safe-area-inset-top)') || '0'),
      right: parseInt(style.getPropertyValue('env(safe-area-inset-right)') || '0'),
      bottom: parseInt(style.getPropertyValue('env(safe-area-inset-bottom)') || '0'),
      left: parseInt(style.getPropertyValue('env(safe-area-inset-left)') || '0'),
    };
  }

  function onKeyboardShow(e: Event) {
    isKeyboardVisible.value = true;
    keyboardHeight.value = (e as CustomEvent).detail?.keyboardHeight || 300;
  }

  function onKeyboardHide() {
    isKeyboardVisible.value = false;
    keyboardHeight.value = 0;
  }

  function onViewportResize() {
    if (window.visualViewport) {
      const heightDiff = window.innerHeight - window.visualViewport.height;
      isKeyboardVisible.value = heightDiff > 150;
      keyboardHeight.value = heightDiff > 150 ? heightDiff : 0;
    }
  }

  onMounted(() => {
    detectPlatform();
    window.addEventListener('keyboardDidShow', onKeyboardShow);
    window.addEventListener('keyboardDidHide', onKeyboardHide);
    window.visualViewport?.addEventListener('resize', onViewportResize);
  });

  onUnmounted(() => {
    window.removeEventListener('keyboardDidShow', onKeyboardShow);
    window.removeEventListener('keyboardDidHide', onKeyboardHide);
    window.visualViewport?.removeEventListener('resize', onViewportResize);
  });

  return {
    platform, isTauri, isMobile, isDesktop, isMac, isWindows, isLinux, isIOS, isAndroid,
    isKeyboardVisible, keyboardHeight, hapticFeedback, getModifierKey, isTouchDevice, getSafeAreaInsets,
  };
}
