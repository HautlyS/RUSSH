<script setup lang="ts">
import { computed } from 'vue';
import { useVisualEffects } from '@/composables/useVisualEffects';
import { Sparkles, Film, Type, Zap, MousePointer, CloudLightning, Layers, RotateCw, SlidersHorizontal, Menu, ListOrdered } from 'lucide-vue-next';

const {
  visualEffects,
  effectsEnabled,
  prefersReducedMotion,
  updateVisualEffects,
  toggleGlobalEffects,
} = useVisualEffects();

// Toggle individual effect
function toggleEffect(effectKey: string) {
  const effect = (visualEffects.value as Record<string, any>)[effectKey];
  if (effect && typeof effect === 'object' && 'enabled' in effect) {
    updateVisualEffects({
      [effectKey]: {
        ...effect,
        enabled: !effect.enabled,
      },
    } as any);
  }
}

// Update effect setting
function updateEffectSetting(effectKey: string, settingKey: string, value: any) {
  const effect = (visualEffects.value as Record<string, any>)[effectKey];
  if (effect && typeof effect === 'object') {
    updateVisualEffects({
      [effectKey]: {
        ...effect,
        [settingKey]: value,
      },
    } as any);
  }
}

const effects = computed(() => [
  {
    key: 'clickSpark',
    name: 'Click Sparks',
    description: 'Animated sparks on click interactions',
    icon: Sparkles,
    enabled: visualEffects.value.clickSpark.enabled,
    settings: [
      { key: 'sparkCount', label: 'Spark Count', type: 'range', min: 4, max: 16, value: visualEffects.value.clickSpark.sparkCount },
      { key: 'sparkRadius', label: 'Radius', type: 'range', min: 10, max: 40, value: visualEffects.value.clickSpark.sparkRadius },
    ],
  },
  {
    key: 'noise',
    name: 'Film Grain',
    description: 'Subtle noise overlay for cinematic feel',
    icon: Film,
    enabled: visualEffects.value.noise.enabled,
    settings: [
      { key: 'alpha', label: 'Intensity', type: 'range', min: 1, max: 20, value: visualEffects.value.noise.alpha },
    ],
  },
  {
    key: 'decryptedText',
    name: 'Decrypted Text',
    description: 'Cyberpunk-style text reveal animations',
    icon: Type,
    enabled: visualEffects.value.decryptedText.enabled,
    settings: [
      { key: 'speed', label: 'Speed', type: 'range', min: 20, max: 100, value: visualEffects.value.decryptedText.speed },
    ],
  },
  {
    key: 'electricBorder',
    name: 'Electric Borders',
    description: 'Animated glowing borders on cards',
    icon: Zap,
    enabled: visualEffects.value.electricBorder.enabled,
    settings: [
      { key: 'speed', label: 'Speed', type: 'range', min: 0.2, max: 2, step: 0.1, value: visualEffects.value.electricBorder.speed },
      { key: 'chaos', label: 'Chaos', type: 'range', min: 0.1, max: 1, step: 0.1, value: visualEffects.value.electricBorder.chaos },
    ],
  },
  {
    key: 'magnet',
    name: 'Magnetic Cursor',
    description: 'Elements attract to cursor on hover',
    icon: MousePointer,
    enabled: visualEffects.value.magnet.enabled,
    settings: [
      { key: 'magnetStrength', label: 'Strength', type: 'range', min: 1, max: 8, value: visualEffects.value.magnet.magnetStrength },
    ],
  },
  {
    key: 'lightning',
    name: 'Lightning Background',
    description: 'WebGL lightning effect (high performance)',
    icon: CloudLightning,
    enabled: visualEffects.value.lightning.enabled,
    settings: [
      { key: 'intensity', label: 'Intensity', type: 'range', min: 0.1, max: 1, step: 0.1, value: visualEffects.value.lightning.intensity },
      { key: 'speed', label: 'Speed', type: 'range', min: 0.2, max: 2, step: 0.1, value: visualEffects.value.lightning.speed },
    ],
  },
  {
    key: 'gradualBlur',
    name: 'Gradual Blur',
    description: 'Smooth blur transitions on scroll areas',
    icon: Layers,
    enabled: visualEffects.value.gradualBlur.enabled,
    settings: [
      { key: 'strength', label: 'Strength', type: 'range', min: 1, max: 5, value: visualEffects.value.gradualBlur.strength },
    ],
  },
  {
    key: 'rotatingText',
    name: 'Rotating Text',
    description: 'Animated text carousel effects',
    icon: RotateCw,
    enabled: visualEffects.value.rotatingText.enabled,
    settings: [
      { key: 'rotationInterval', label: 'Interval (ms)', type: 'range', min: 1000, max: 5000, step: 500, value: visualEffects.value.rotatingText.rotationInterval },
    ],
  },
  {
    key: 'elasticSlider',
    name: 'Elastic Sliders',
    description: 'Physics-based slider animations',
    icon: SlidersHorizontal,
    enabled: visualEffects.value.elasticSlider.enabled,
    settings: [],
  },
  {
    key: 'flowingMenu',
    name: 'Flowing Menu',
    description: 'Animated menu with marquee effect',
    icon: Menu,
    enabled: visualEffects.value.flowingMenu.enabled,
    settings: [],
  },
  {
    key: 'stepper',
    name: 'Animated Stepper',
    description: 'Multi-step wizard with animations',
    icon: ListOrdered,
    enabled: visualEffects.value.stepper.enabled,
    settings: [],
  },
]);
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-1">Visual Effects</h2>
      <p class="text-sm text-gray-500 dark:text-gray-400">
        Customize animations and visual effects throughout the application
      </p>
    </div>

    <!-- System Reduced Motion Warning -->
    <div 
      v-if="prefersReducedMotion" 
      class="p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg"
    >
      <p class="text-sm text-yellow-800 dark:text-yellow-200">
        Your system has reduced motion enabled. Visual effects are automatically disabled to respect your preference.
      </p>
    </div>

    <!-- Global Toggle -->
    <div class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
      <div>
        <h3 class="font-medium text-gray-900 dark:text-white">Enable All Effects</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400">Master toggle for all visual effects</p>
      </div>
      <button
        @click="toggleGlobalEffects"
        :class="[
          'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
          visualEffects.globalEnabled ? 'bg-green-500' : 'bg-gray-300 dark:bg-gray-600'
        ]"
      >
        <span
          :class="[
            'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
            visualEffects.globalEnabled ? 'translate-x-6' : 'translate-x-1'
          ]"
        />
      </button>
    </div>

    <!-- Reduced Motion Override -->
    <div class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
      <div>
        <h3 class="font-medium text-gray-900 dark:text-white">Reduce Motion</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400">Disable animations for accessibility</p>
      </div>
      <button
        @click="updateVisualEffects({ reducedMotion: !visualEffects.reducedMotion })"
        :class="[
          'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
          visualEffects.reducedMotion ? 'bg-green-500' : 'bg-gray-300 dark:bg-gray-600'
        ]"
      >
        <span
          :class="[
            'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
            visualEffects.reducedMotion ? 'translate-x-6' : 'translate-x-1'
          ]"
        />
      </button>
    </div>

    <!-- Individual Effects -->
    <div class="space-y-4">
      <h3 class="font-medium text-gray-900 dark:text-white">Individual Effects</h3>
      
      <div 
        v-for="effect in effects" 
        :key="effect.key"
        class="p-4 bg-gray-50 dark:bg-gray-800 rounded-lg space-y-3"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="p-2 bg-gray-200 dark:bg-gray-700 rounded-lg">
              <component :is="effect.icon" class="w-5 h-5 text-gray-600 dark:text-gray-300" />
            </div>
            <div>
              <h4 class="font-medium text-gray-900 dark:text-white">{{ effect.name }}</h4>
              <p class="text-sm text-gray-500 dark:text-gray-400">{{ effect.description }}</p>
            </div>
          </div>
          <button
            @click="toggleEffect(effect.key)"
            :disabled="!effectsEnabled"
            :class="[
              'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
              effect.enabled && effectsEnabled ? 'bg-green-500' : 'bg-gray-300 dark:bg-gray-600',
              !effectsEnabled && 'opacity-50 cursor-not-allowed'
            ]"
          >
            <span
              :class="[
                'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                effect.enabled && effectsEnabled ? 'translate-x-6' : 'translate-x-1'
              ]"
            />
          </button>
        </div>

        <!-- Effect Settings -->
        <div 
          v-if="effect.settings.length > 0 && effect.enabled && effectsEnabled" 
          class="pl-12 space-y-3"
        >
          <div v-for="setting in effect.settings" :key="setting.key" class="space-y-1">
            <div class="flex items-center justify-between">
              <label class="text-sm text-gray-600 dark:text-gray-400">{{ setting.label }}</label>
              <span class="text-sm font-mono text-gray-500">{{ setting.value }}</span>
            </div>
            <input
              type="range"
              :min="setting.min"
              :max="setting.max"
              :step="setting.step || 1"
              :value="setting.value"
              @input="(e) => updateEffectSetting(effect.key, setting.key, Number((e.target as HTMLInputElement).value))"
              class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
