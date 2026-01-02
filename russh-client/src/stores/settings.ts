/**
 * Settings store - manages application settings
 */

import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '@/types/settings';
import { defaultSettings } from '@/types/settings';

export const useSettingsStore = defineStore('settings', () => {
  // State
  const settings = ref<AppSettings>(structuredClone(defaultSettings));
  const isLoading = ref(false);
  const isDirty = ref(false);

  // Actions
  async function loadSettings() {
    isLoading.value = true;
    try {
      const saved = await invoke<AppSettings | null>('settings_load');
      if (saved) {
        // Merge with defaults to ensure all fields exist
        settings.value = mergeSettings(defaultSettings, saved);
      }
      isDirty.value = false;
    } catch (e) {
      console.error('Failed to load settings:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function saveSettings() {
    try {
      await invoke('settings_save', { settings: settings.value });
      isDirty.value = false;
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }

  function updateSettings(partial: Partial<AppSettings>) {
    settings.value = {
      ...settings.value,
      ...partial,
      general: { ...settings.value.general, ...partial.general },
      terminal: { ...settings.value.terminal, ...partial.terminal },
      appearance: { ...settings.value.appearance, ...partial.appearance },
      keyboard: { ...settings.value.keyboard, ...partial.keyboard },
      notifications: { ...settings.value.notifications, ...partial.notifications },
      visualEffects: { ...settings.value.visualEffects, ...partial.visualEffects },
    };
    isDirty.value = true;
  }

  function updateSetting<K extends keyof AppSettings>(
    category: K,
    key: keyof AppSettings[K],
    value: AppSettings[K][keyof AppSettings[K]]
  ) {
    (settings.value[category] as Record<string, unknown>)[key as string] = value;
    isDirty.value = true;
  }

  function resetToDefaults() {
    settings.value = structuredClone(defaultSettings);
    isDirty.value = true;
    saveSettings();
  }

  function resetCategory<K extends keyof AppSettings>(category: K) {
    settings.value[category] = structuredClone(defaultSettings[category]);
    isDirty.value = true;
    saveSettings();
  }

  async function exportSettings(): Promise<string> {
    return JSON.stringify(settings.value, null, 2);
  }

  async function importSettings(jsonData: string) {
    try {
      const imported = JSON.parse(jsonData) as AppSettings;
      settings.value = mergeSettings(defaultSettings, imported);
      isDirty.value = true;
      await saveSettings();
    } catch (e) {
      throw new Error('Invalid settings format');
    }
  }

  // Helper to merge settings with defaults
  function mergeSettings(defaults: AppSettings, saved: Partial<AppSettings>): AppSettings {
    return {
      general: { ...defaults.general, ...saved.general },
      terminal: { ...defaults.terminal, ...saved.terminal },
      appearance: { ...defaults.appearance, ...saved.appearance },
      keyboard: { 
        ...defaults.keyboard, 
        ...saved.keyboard,
        shortcuts: { ...defaults.keyboard.shortcuts, ...saved.keyboard?.shortcuts }
      },
      notifications: { ...defaults.notifications, ...saved.notifications },
      visualEffects: {
        ...defaults.visualEffects,
        ...saved.visualEffects,
        clickSpark: { ...defaults.visualEffects.clickSpark, ...saved.visualEffects?.clickSpark },
        noise: { ...defaults.visualEffects.noise, ...saved.visualEffects?.noise },
        decryptedText: { ...defaults.visualEffects.decryptedText, ...saved.visualEffects?.decryptedText },
        electricBorder: { ...defaults.visualEffects.electricBorder, ...saved.visualEffects?.electricBorder },
        magnet: { ...defaults.visualEffects.magnet, ...saved.visualEffects?.magnet },
        lightning: { ...defaults.visualEffects.lightning, ...saved.visualEffects?.lightning },
        gradualBlur: { ...defaults.visualEffects.gradualBlur, ...saved.visualEffects?.gradualBlur },
        rotatingText: { ...defaults.visualEffects.rotatingText, ...saved.visualEffects?.rotatingText },
        elasticSlider: { ...defaults.visualEffects.elasticSlider, ...saved.visualEffects?.elasticSlider },
        flowingMenu: { ...defaults.visualEffects.flowingMenu, ...saved.visualEffects?.flowingMenu },
        stepper: { ...defaults.visualEffects.stepper, ...saved.visualEffects?.stepper },
      },
    };
  }

  // Auto-save on changes (debounced)
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  watch(settings, () => {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      if (isDirty.value) {
        saveSettings();
      }
    }, 1000);
  }, { deep: true });

  return {
    // State
    settings,
    isLoading,
    isDirty,
    // Actions
    loadSettings,
    saveSettings,
    updateSettings,
    updateSetting,
    resetToDefaults,
    resetCategory,
    exportSettings,
    importSettings,
  };
});
