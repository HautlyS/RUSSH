# Visual Effects Guide

RUSSH includes a rich set of visual effects to enhance the user experience. All effects respect system accessibility preferences and can be individually toggled.

## Available Effects

### Click Spark

Animated sparks that emanate from click locations.

**Settings:**
| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `enabled` | boolean | true | Enable/disable effect |
| `sparkColor` | string | '#27FF64' | Spark color (hex) |
| `sparkCount` | number | 8 | Number of sparks |
| `sparkSize` | number | 10 | Spark line length |
| `sparkRadius` | number | 15 | Spread radius |
| `duration` | number | 400 | Animation duration (ms) |

**Usage:**
```vue
<ClickSpark
  spark-color="#27FF64"
  :spark-count="8"
  :spark-size="10"
  :spark-radius="15"
  :duration="400"
>
  <YourContent />
</ClickSpark>
```

### Noise Overlay

Subtle film grain effect for texture.

**Settings:**
| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `enabled` | boolean | false | Enable/disable effect |
| `alpha` | number | 10 | Noise opacity (0-255) |
| `refreshInterval` | number | 2 | Frame refresh rate |
| `mixBlendMode` | string | 'normal' | CSS blend mode |

**Usage:**
```vue
<Noise
  :pattern-alpha="10"
  :pattern-refresh-interval="2"
  mix-blend-mode="overlay"
/>
```

### Lightning

WebGL-powered lightning effect for backgrounds.

**Settings:**
| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `enabled` | boolean | true | Enable/disable effect |
| `hue` | number | 230 | Color hue (0-360) |
| `intensity` | number | 1 | Brightness multiplier |
| `speed` | number | 1 | Animation speed |
| `locations` | string[] | ['terminal'] | Where to show |

**Locations:** `terminal`, `p2p`, `dashboard`

**Usage:**
```vue
<Lightning
  :hue="230"
  :intensity="1"
  :speed="1"
  :size="1"
/>
```

### Electric Border

Animated electric border effect.

**Settings:**
| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `enabled` | boolean | true | Enable/disable effect |
| `color` | string | '#7df9ff' | Border color |
| `speed` | number | 1 | Animation speed |
| `chaos` | number | 0.5 | Turbulence amount |
| `thickness` | number | 2 | Border thickness |

**Usage:**
```vue
<ElectricBorder
  color="#7df9ff"
  :speed="1.2"
  :chaos="0.5"
  :thickness="2"
>
  <YourContent />
</ElectricBorder>
```

### Decrypted Text

Matrix-style text reveal animation.

**Settings:**
| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `enabled` | boolean | true | Enable/disable effect |
| `speed` | number | 50 | Character change speed (ms) |
| `maxIterations` | number | 10 | Scramble iterations |
| `sequential` | boolean | false | Reveal sequentially |

**Usage:**
```vue
<DecryptedText
  text="Hello World"
  :speed="50"
  :max-iterations="10"
  :sequential="true"
  animate-on="view"
  reveal-direction="start"
/>
```

### Rotating Text

Animated text rotation with stagger effects.

**Settings:**
| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `enabled` | boolean | true | Enable/disable effect |
| `rotationInterval` | number | 2000 | Time between rotations (ms) |
| `staggerDuration` | number | 0.05 | Stagger delay per character |

**Usage:**
```vue
<RotatingText
  :texts="['First', 'Second', 'Third']"
  :rotation-interval="2000"
  :stagger-duration="0.05"
/>
```

### Magnet

Magnetic cursor attraction effect.

**Settings:**
| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `enabled` | boolean | true | Enable/disable effect |
| `padding` | number | 100 | Activation distance |
| `magnetStrength` | number | 2 | Pull strength |

**Usage:**
```vue
<Magnet
  :padding="100"
  :magnet-strength="2"
>
  <button>Hover me</button>
</Magnet>
```

### Gradual Blur

Gradient blur effect for scroll areas.

**Settings:**
| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `enabled` | boolean | true | Enable/disable effect |
| `height` | string | '6rem' | Blur zone height |
| `strength` | number | 2 | Blur intensity |

**Usage:**
```vue
<GradualBlur
  position="top"
  height="6rem"
  :strength="2"
  curve="bezier"
/>
```

## Global Settings

### Enabling/Disabling All Effects

```typescript
const { toggleGlobalEffects, effectsEnabled } = useVisualEffects();

// Toggle all effects
toggleGlobalEffects();

// Check if effects are enabled
if (effectsEnabled.value) {
  // Effects are active
}
```

### Reduced Motion Support

RUSSH automatically respects the system's reduced motion preference:

```typescript
const { prefersReducedMotion, setReducedMotion } = useVisualEffects();

// Check system preference
if (prefersReducedMotion.value) {
  // User prefers reduced motion
}

// Override manually
setReducedMotion(true);
```

### Checking Individual Effects

```typescript
const {
  isClickSparkEnabled,
  isNoiseEnabled,
  isLightningEnabled,
  isElectricBorderEnabled,
  isDecryptedTextEnabled,
  isRotatingTextEnabled,
  isMagnetEnabled,
  isGradualBlurEnabled,
} = useVisualEffects();

// Location-specific check
const isLightningEnabled = isLightningEnabledFor('terminal');
```

## Settings UI

Effects can be configured in **Settings → Visual Effects**:

1. **Global Toggle** - Enable/disable all effects
2. **Reduced Motion** - Override system preference
3. **Individual Toggles** - Enable/disable specific effects
4. **Effect Settings** - Customize each effect's parameters

## Performance Considerations

### WebGL Effects (Lightning)

- Uses GPU acceleration
- Falls back gracefully if WebGL unavailable
- Automatically pauses when tab is hidden

### Canvas Effects (ClickSpark, Noise)

- Lightweight canvas rendering
- Efficient animation frame management
- Proper cleanup on unmount

### CSS Effects (ElectricBorder, GradualBlur)

- Hardware-accelerated CSS transforms
- Minimal JavaScript overhead
- Uses `will-change` for optimization

## Best Practices

1. **Respect User Preferences**
   - Always check `effectsEnabled` before rendering
   - Support `prefers-reduced-motion`

2. **Clean Up Resources**
   - Cancel animation frames in `onUnmounted`
   - Remove event listeners
   - Dispose WebGL contexts

3. **Conditional Rendering**
   - Use `v-if` to completely remove disabled effects
   - Don't just hide with CSS

4. **Performance**
   - Limit effects on mobile devices
   - Reduce intensity for battery savings
   - Use `shallowRef` for effect state

## Troubleshooting

### Effects Not Showing

1. Check if globally enabled
2. Verify individual effect is enabled
3. Check reduced motion setting
4. Verify component is mounted

### Performance Issues

1. Reduce effect intensity
2. Disable WebGL effects
3. Lower noise refresh rate
4. Disable on mobile

### Visual Glitches

1. Check browser WebGL support
2. Update graphics drivers
3. Try different blend modes
4. Reduce chaos/turbulence values
