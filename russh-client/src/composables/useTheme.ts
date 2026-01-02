/**
 * Theme composable - handles theme switching and system detection
 */

import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';

export function useTheme() {
  const settingsStore = useSettingsStore();
  const systemTheme = ref<'light' | 'dark'>('dark');
  let mediaQuery: MediaQueryList | null = null;
  let mediaQueryHandler: ((e: MediaQueryListEvent) => void) | null = null;

  const theme = computed(() => settingsStore.settings.appearance.theme);

  const effectiveTheme = computed(() => {
    const setting = settingsStore.settings.appearance.theme;
    if (setting === 'system') {
      return systemTheme.value;
    }
    return setting as 'light' | 'dark';
  });

  const isDark = computed(() => effectiveTheme.value === 'dark');

  function applyTheme() {
    const root = document.documentElement;
    if (isDark.value) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
    
    // Apply accent color as CSS variable
    applyAccentColor(settingsStore.settings.appearance.accentColor);
  }

  function applyAccentColor(color: string) {
    const root = document.documentElement;
    
    // Convert hex to RGB for CSS variables
    const hex = color.replace('#', '');
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    
    // Generate color scale (simplified)
    root.style.setProperty('--accent-500', `${r} ${g} ${b}`);
    root.style.setProperty('--accent-400', `${Math.min(r + 30, 255)} ${Math.min(g + 30, 255)} ${Math.min(b + 30, 255)}`);
    root.style.setProperty('--accent-600', `${Math.max(r - 30, 0)} ${Math.max(g - 30, 0)} ${Math.max(b - 30, 0)}`);
  }

  function detectSystemTheme() {
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    systemTheme.value = mediaQuery.matches ? 'dark' : 'light';
    
    mediaQueryHandler = (e: MediaQueryListEvent) => {
      systemTheme.value = e.matches ? 'dark' : 'light';
    };
    mediaQuery.addEventListener('change', mediaQueryHandler);
  }

  function setTheme(theme: 'light' | 'dark' | 'system') {
    settingsStore.updateSetting('appearance', 'theme', theme);
  }

  function toggleTheme() {
    const newTheme = isDark.value ? 'light' : 'dark';
    setTheme(newTheme);
  }

  // Watch for theme changes
  watch(effectiveTheme, applyTheme);
  watch(() => settingsStore.settings.appearance.accentColor, applyAccentColor);

  onMounted(() => {
    detectSystemTheme();
    applyTheme();
  });

  onUnmounted(() => {
    if (mediaQuery && mediaQueryHandler) {
      mediaQuery.removeEventListener('change', mediaQueryHandler);
    }
  });

  return {
    theme,
    effectiveTheme,
    isDark,
    systemTheme,
    setTheme,
    toggleTheme,
  };
}
