# Design Document: RUSSH Cross-Device SSH Client - Visual Effects Integration

## Overview

This design document extends the RUSSH SSH client with premium visual effects using the extra components library. The integration aims to create a modern, polished user experience while maintaining performance and accessibility. Each visual effect component is strategically placed to enhance user interaction without overwhelming the interface.

## Extra Components Catalog

### Component Classification

| Component | Type | Purpose | Performance Impact |
|-----------|------|---------|-------------------|
| ClickSpark | Mouse Effect | Click feedback animation | Low |
| DecryptedText | Text Animation | Cyberpunk text reveal effect | Low |
| ElasticSlider | Input Control | Animated slider with physics | Low |
| ElectricBorder | Border Effect | Animated glowing border | Medium |
| FlowingMenu | Navigation | Animated menu with marquee | Medium |
| GradualBlur | Background Effect | Gradient blur overlay | Low |
| Lightning | Background VFX | WebGL lightning animation | High |
| Magnet | Mouse Effect | Magnetic cursor attraction | Low |
| Noise | Background Effect | Film grain overlay | Low |
| RotatingText | Text Animation | Rotating text carousel | Low |
| Stepper | UI Control | Multi-step wizard | Low |

## Architecture

### Component Integration Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│                        App.vue (Root)                           │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  ClickSpark (Global click effect wrapper)               │   │
│  │  ┌─────────────────────────────────────────────────┐   │   │
│  │  │  Noise (Subtle film grain overlay)              │   │   │
│  │  │  ┌─────────────────────────────────────────┐   │   │   │
│  │  │  │  AppHeader + AppSidebar + RouterView   │   │   │   │
│  │  │  └─────────────────────────────────────────┘   │   │   │
│  │  └─────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Visual Effects Layer Hierarchy

1. **Base Layer**: Application content
2. **Effect Layer**: Noise overlay (z-index: 1000)
3. **Interaction Layer**: ClickSpark canvas (z-index: 1001)
4. **UI Layer**: Modals, tooltips (z-index: 1100+)

## Components and Interfaces

### 1. Global Effects (App.vue)

#### ClickSpark Integration
- **Location**: Wraps entire application
- **Purpose**: Provides satisfying click feedback throughout the app
- **Configuration**:
  - `sparkColor`: Theme accent color (#27FF64 for dark mode)
  - `sparkCount`: 8 sparks per click
  - `sparkRadius`: 20px
  - `duration`: 400ms

#### Noise Overlay
- **Location**: Fixed overlay on entire viewport
- **Purpose**: Adds subtle film grain for premium aesthetic
- **Configuration**:
  - `patternAlpha`: 5 (very subtle)
  - `patternRefreshInterval`: 3 (performance optimized)
  - `mixBlendMode`: "overlay"

### 2. Dashboard View (DashboardView.vue)

#### DecryptedText for Welcome Message
- **Location**: Main heading "Welcome to RUSSH"
- **Purpose**: Cyberpunk-style text reveal on page load
- **Configuration**:
  - `animateOn`: "view"
  - `speed`: 40
  - `sequential`: true
  - `revealDirection`: "start"

#### RotatingText for Status Messages
- **Location**: Below welcome message
- **Purpose**: Rotating tips/status messages
- **Configuration**:
  - `texts`: ["Secure connections", "Cross-device sync", "P2P networking", "Fast transfers"]
  - `rotationInterval`: 3000
  - `staggerDuration`: 0.03

#### ElectricBorder for Quick Action Cards
- **Location**: "New Connection" and "P2P Connections" action cards
- **Purpose**: Highlight primary actions with animated border
- **Configuration**:
  - `color`: "#27FF64" (accent green)
  - `speed`: 0.8
  - `chaos`: 0.3
  - `thickness`: 2

### 3. Connections View (ConnectionsView.vue)

#### Magnet Effect for Connection Cards
- **Location**: Each ConnectionCard component
- **Purpose**: Subtle magnetic attraction on hover
- **Configuration**:
  - `padding`: 80
  - `magnetStrength`: 4
  - `activeTransition`: "transform 0.2s ease-out"

#### GradualBlur for List Scroll
- **Location**: Top and bottom of connection list
- **Purpose**: Smooth fade effect for scrollable content
- **Configuration**:
  - `position`: "bottom" / "top"
  - `height`: "4rem"
  - `strength`: 2
  - `curve`: "bezier"

### 4. Terminal View (TerminalView.vue)

#### Lightning Background (Optional/Settings)
- **Location**: Behind terminal container
- **Purpose**: Dramatic background effect for terminal
- **Configuration**:
  - `hue`: 230 (blue) or 120 (green matrix style)
  - `intensity`: 0.3 (subtle)
  - `speed`: 0.5
- **Note**: Disabled by default, user-enabled in settings

#### DecryptedText for Connection Status
- **Location**: Terminal toolbar connection info
- **Purpose**: Animated hostname display on connect
- **Configuration**:
  - `animateOn`: "view"
  - `speed`: 30
  - `characters`: "01"

### 5. Settings View (SettingsView.vue)

#### Stepper for Onboarding/Setup Wizard
- **Location**: First-time setup or import wizard
- **Purpose**: Guided multi-step configuration
- **Configuration**:
  - `lockOnComplete`: false
  - Custom step content for each setting category

#### ElasticSlider for Numeric Settings
- **Location**: Terminal font size, opacity, timeout values
- **Purpose**: Satisfying slider interaction
- **Configuration**:
  - `isStepped`: true (for discrete values)
  - `stepSize`: varies by setting

### 6. P2P View (P2PView.vue)

#### Lightning Background
- **Location**: P2P status section background
- **Purpose**: Visual representation of P2P network activity
- **Configuration**:
  - `hue`: 280 (purple)
  - `intensity`: 0.5
  - `speed`: 1

#### ElectricBorder for QR Code
- **Location**: QR code share component
- **Purpose**: Draw attention to shareable QR code
- **Configuration**:
  - `color`: "#7df9ff" (cyan)
  - `speed`: 1.2
  - `chaos`: 0.5

#### RotatingText for Connection Tips
- **Location**: Below peer list
- **Purpose**: Rotating P2P connection tips
- **Configuration**:
  - `texts`: ["Share your QR code", "Direct connections are faster", "NAT traversal enabled"]

### 7. Header Component (AppHeader.vue)

#### DecryptedText for Logo
- **Location**: "RUSSH" text in header
- **Purpose**: Brand identity animation on app load
- **Configuration**:
  - `animateOn`: "view"
  - `speed`: 60
  - `maxIterations`: 15

#### Magnet for Action Buttons
- **Location**: Notification bell, settings icon
- **Purpose**: Subtle magnetic effect on header icons
- **Configuration**:
  - `padding`: 40
  - `magnetStrength`: 5

### 8. Sidebar Component (AppSidebar.vue)

#### FlowingMenu for Main Navigation (Mobile)
- **Location**: Mobile navigation menu
- **Purpose**: Premium menu animation for mobile
- **Configuration**:
  - Custom items with connection icons

#### GradualBlur for Scroll Areas
- **Location**: Connection list scroll container
- **Purpose**: Fade effect at scroll boundaries
- **Configuration**:
  - `position`: "bottom"
  - `height`: "3rem"

### 9. Connection Card Component (ConnectionCard.vue)

#### ElectricBorder on Hover
- **Location**: Card border on hover state
- **Purpose**: Highlight active/hovered connection
- **Configuration**:
  - `color`: Profile color or default accent
  - `speed`: 1
  - `thickness`: 1

#### DecryptedText for Server Name
- **Location**: Connection name on card
- **Purpose**: Reveal animation when card enters view
- **Configuration**:
  - `animateOn`: "view"
  - `speed`: 50

### 10. Command Palette (CommandPalette.vue)

#### GradualBlur for Backdrop
- **Location**: Behind command palette modal
- **Purpose**: Smooth blur transition
- **Configuration**:
  - `target`: "page"
  - `strength`: 4

## Data Models

### Visual Effects Settings Schema

```typescript
interface VisualEffectsSettings {
  // Global effects
  clickSpark: {
    enabled: boolean;
    color: string;
    sparkCount: number;
  };
  noise: {
    enabled: boolean;
    alpha: number;
    refreshInterval: number;
  };
  
  // Component-specific
  decryptedText: {
    enabled: boolean;
    speed: number;
  };
  electricBorder: {
    enabled: boolean;
    intensity: 'subtle' | 'normal' | 'intense';
  };
  lightning: {
    enabled: boolean;
    locations: ('terminal' | 'p2p' | 'dashboard')[];
  };
  
  // Performance
  reducedMotion: boolean;
  gpuAcceleration: boolean;
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Visual Effects Respect Reduced Motion
*For any* user with `prefers-reduced-motion: reduce` system setting, all animated visual effects SHALL be disabled or use instant transitions.
**Validates: Requirements 13.1 (Accessibility)**

### Property 2: Click Spark Coordinates Match Click Position
*For any* click event within the ClickSpark wrapper, the spark animation origin SHALL match the click coordinates within 1px tolerance.
**Validates: Requirements 2.1 (Vue Frontend Architecture)**

### Property 3: Visual Effects Settings Persistence
*For any* valid VisualEffectsSettings object, saving then loading SHALL produce an equivalent settings object.
**Validates: Requirements 8.7 (Settings Round-Trip)**

### Property 4: Performance Budget Compliance
*For any* combination of enabled visual effects, the total frame time SHALL not exceed 16ms (60fps) on target hardware.
**Validates: Requirements 2.1 (Vue Frontend Architecture)**

### Property 5: Theme Color Propagation
*For any* theme change event, all visual effect components using theme colors SHALL update within the same render cycle.
**Validates: Requirements 6.1, 6.2 (Theme Management)**

## Error Handling

### WebGL Fallback
- Lightning component requires WebGL
- Fallback: Static gradient background
- Detection: Check `canvas.getContext('webgl')` availability

### Performance Degradation
- Monitor frame rate during effects
- Auto-disable effects if FPS drops below 30
- User notification when effects are auto-disabled

### Animation Cleanup
- All animation frames must be cancelled on component unmount
- ResizeObservers must be disconnected
- Event listeners must be removed

## Testing Strategy

### Unit Tests
- Visual effects settings store actions
- Theme color computation for effects
- Reduced motion detection

### Property-Based Tests
- Settings serialization round-trip
- Click coordinate accuracy
- Theme color propagation timing

### Integration Tests
- Effect components render without errors
- Effects respond to settings changes
- Performance metrics within budget

### Visual Regression Tests
- Screenshot comparison for effect states
- Animation keyframe verification

## Implementation Priority

### Phase 1: Global Effects (High Impact, Low Risk)
1. ClickSpark wrapper in App.vue
2. Noise overlay with settings toggle
3. Visual effects settings in store

### Phase 2: Text Animations (Medium Impact)
1. DecryptedText for headings
2. RotatingText for status messages
3. Accessibility compliance

### Phase 3: Interactive Effects (High Impact)
1. ElectricBorder for cards and actions
2. Magnet effect for buttons
3. ElasticSlider for settings

### Phase 4: Background Effects (Medium Impact, Higher Risk)
1. Lightning for P2P view
2. GradualBlur for scroll areas
3. Performance monitoring

### Phase 5: Navigation Effects (Lower Priority)
1. FlowingMenu for mobile
2. Stepper for wizards
3. Final polish and optimization

## Performance Considerations

### GPU Optimization
- Use `will-change` CSS property sparingly
- Prefer `transform` and `opacity` for animations
- Batch DOM reads/writes

### Memory Management
- Reuse canvas contexts where possible
- Limit concurrent animations
- Implement object pooling for particles

### Battery Impact (Mobile)
- Reduce animation frequency on battery
- Disable WebGL effects on low battery
- Respect system power saving mode
