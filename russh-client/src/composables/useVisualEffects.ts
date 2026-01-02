/**
 * Visual Effects Composable
 * Provides reactive access to visual effects settings with reduced motion support
 */

import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import type { VisualEffectsSettings } from '@/types/visualEffects';

export function useVisualEffects() {
  const settingsStore = useSettingsStore();
  
  // System reduced motion preference
  const prefersReducedMotion = ref(false);
  let mediaQuery: MediaQueryList | null = null;

  // Check system preference for reduced motion
  const checkReducedMotion = () => {
    if (typeof window !== 'undefined') {
      mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
      prefersReducedMotion.value = mediaQuery.matches;
    }
  };

  const handleMotionChange = (e: MediaQueryListEvent) => {
    prefersReducedMotion.value = e.matches;
  };

  onMounted(() => {
    checkReducedMotion();
    mediaQuery?.addEventListener('change', handleMotionChange);
  });

  onUnmounted(() => {
    mediaQuery?.removeEventListener('change', handleMotionChange);
  });

  // Computed settings with reduced motion override
  const visualEffects = computed(() => settingsStore.settings.visualEffects);

  // Whether effects should be active (respects both user setting and system preference)
  const effectsEnabled = computed(() => {
    return visualEffects.value.globalEnabled && 
           !visualEffects.value.reducedMotion && 
           !prefersReducedMotion.value;
  });

  // Individual effect availability checks
  const isClickSparkEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.clickSpark.enabled
  );

  const isNoiseEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.noise.enabled
  );

  const isDecryptedTextEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.decryptedText.enabled
  );

  const isElectricBorderEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.electricBorder.enabled
  );

  const isMagnetEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.magnet.enabled
  );

  const isLightningEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.lightning.enabled
  );

  const isGradualBlurEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.gradualBlur.enabled
  );

  const isRotatingTextEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.rotatingText.enabled
  );

  const isElasticSliderEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.elasticSlider.enabled
  );

  const isFlowingMenuEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.flowingMenu.enabled
  );

  const isStepperEnabled = computed(() => 
    effectsEnabled.value && visualEffects.value.stepper.enabled
  );

  // Check if lightning is enabled for a specific location
  const isLightningEnabledFor = (location: 'terminal' | 'p2p' | 'dashboard') => {
    return computed(() => 
      isLightningEnabled.value && 
      visualEffects.value.lightning.locations.includes(location)
    );
  };

  // Update functions
  const updateVisualEffects = (partial: Partial<VisualEffectsSettings>) => {
    settingsStore.updateSettings({
      visualEffects: {
        ...visualEffects.value,
        ...partial,
      },
    });
  };

  const toggleEffect = (effectName: keyof VisualEffectsSettings) => {
    const effect = visualEffects.value[effectName];
    if (typeof effect === 'object' && 'enabled' in effect) {
      updateVisualEffects({
        [effectName]: {
          ...effect,
          enabled: !effect.enabled,
        },
      } as Partial<VisualEffectsSettings>);
    }
  };

  const toggleGlobalEffects = () => {
    updateVisualEffects({
      globalEnabled: !visualEffects.value.globalEnabled,
    });
  };

  const setReducedMotion = (value: boolean) => {
    updateVisualEffects({
      reducedMotion: value,
    });
  };

  // Get theme-aware colors
  const getThemeAwareColor = (lightColor: string, darkColor: string) => {
    const isDark = settingsStore.settings.appearance.theme === 'dark' ||
      (settingsStore.settings.appearance.theme === 'system' && 
       window.matchMedia('(prefers-color-scheme: dark)').matches);
    return isDark ? darkColor : lightColor;
  };

  // Get accent color from settings
  const accentColor = computed(() => settingsStore.settings.appearance.accentColor);

  return {
    // Settings
    visualEffects,
    prefersReducedMotion,
    effectsEnabled,
    accentColor,

    // Individual effect checks
    isClickSparkEnabled,
    isNoiseEnabled,
    isDecryptedTextEnabled,
    isElectricBorderEnabled,
    isMagnetEnabled,
    isLightningEnabled,
    isGradualBlurEnabled,
    isRotatingTextEnabled,
    isElasticSliderEnabled,
    isFlowingMenuEnabled,
    isStepperEnabled,
    isLightningEnabledFor,

    // Actions
    updateVisualEffects,
    toggleEffect,
    toggleGlobalEffects,
    setReducedMotion,
    getThemeAwareColor,
  };
}
