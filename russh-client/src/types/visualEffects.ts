/**
 * Visual Effects Settings Types
 * Configuration for all visual effect components in the application
 */

export interface ClickSparkSettings {
  enabled: boolean;
  color: string;
  sparkCount: number;
  sparkSize: number;
  sparkRadius: number;
  duration: number;
}

export interface NoiseSettings {
  enabled: boolean;
  alpha: number;
  refreshInterval: number;
  mixBlendMode: string;
}

export interface DecryptedTextSettings {
  enabled: boolean;
  speed: number;
  maxIterations: number;
  sequential: boolean;
}

export interface ElectricBorderSettings {
  enabled: boolean;
  color: string;
  speed: number;
  chaos: number;
  thickness: number;
}

export interface MagnetSettings {
  enabled: boolean;
  padding: number;
  magnetStrength: number;
}

export interface LightningSettings {
  enabled: boolean;
  hue: number;
  intensity: number;
  speed: number;
  locations: ('terminal' | 'p2p' | 'dashboard')[];
}

export interface GradualBlurSettings {
  enabled: boolean;
  strength: number;
  height: string;
}

export interface RotatingTextSettings {
  enabled: boolean;
  rotationInterval: number;
  staggerDuration: number;
}

export interface ElasticSliderSettings {
  enabled: boolean;
}

export interface FlowingMenuSettings {
  enabled: boolean;
}

export interface StepperSettings {
  enabled: boolean;
}

export interface VisualEffectsSettings {
  // Global toggle
  globalEnabled: boolean;
  reducedMotion: boolean;
  gpuAcceleration: boolean;

  // Individual effect settings
  clickSpark: ClickSparkSettings;
  noise: NoiseSettings;
  decryptedText: DecryptedTextSettings;
  electricBorder: ElectricBorderSettings;
  magnet: MagnetSettings;
  lightning: LightningSettings;
  gradualBlur: GradualBlurSettings;
  rotatingText: RotatingTextSettings;
  elasticSlider: ElasticSliderSettings;
  flowingMenu: FlowingMenuSettings;
  stepper: StepperSettings;
}

export const defaultVisualEffectsSettings: VisualEffectsSettings = {
  globalEnabled: true,
  reducedMotion: false,
  gpuAcceleration: true,

  clickSpark: {
    enabled: true,
    color: '#27FF64',
    sparkCount: 8,
    sparkSize: 10,
    sparkRadius: 20,
    duration: 400,
  },

  noise: {
    enabled: true,
    alpha: 5,
    refreshInterval: 3,
    mixBlendMode: 'overlay',
  },

  decryptedText: {
    enabled: true,
    speed: 50,
    maxIterations: 10,
    sequential: true,
  },

  electricBorder: {
    enabled: true,
    color: '#27FF64',
    speed: 0.8,
    chaos: 0.3,
    thickness: 2,
  },

  magnet: {
    enabled: true,
    padding: 80,
    magnetStrength: 4,
  },

  lightning: {
    enabled: false, // Disabled by default for performance
    hue: 230,
    intensity: 0.3,
    speed: 0.5,
    locations: [],
  },

  gradualBlur: {
    enabled: true,
    strength: 2,
    height: '4rem',
  },

  rotatingText: {
    enabled: true,
    rotationInterval: 3000,
    staggerDuration: 0.03,
  },

  elasticSlider: {
    enabled: true,
  },

  flowingMenu: {
    enabled: true,
  },

  stepper: {
    enabled: true,
  },
};
