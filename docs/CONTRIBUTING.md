# Contributing to RUSSH

Thank you for your interest in contributing to RUSSH! This guide will help you get started.

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something great together.

## Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.70+
- pnpm (recommended)
- Git

### Development Setup

```bash
# Fork and clone the repository
git clone https://github.com/hautlyS/russh.git
cd russh

# Install frontend dependencies
cd russh-client
pnpm install

# Start development server
pnpm tauri dev
```

### Project Structure

```
russh/
├── russh-ssh/          # Core Rust SSH library
├── russh-ssh-cli/      # CLI tool
├── russh-client/       # Tauri + Vue frontend
│   ├── src/
│   │   ├── components/ # Vue components
│   │   ├── composables/# Vue composables
│   │   ├── stores/     # Pinia stores
│   │   ├── types/      # TypeScript types
│   │   └── views/      # Page views
│   └── src-tauri/      # Tauri backend
└── docs/               # Documentation
```

## Development Workflow

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Code refactoring

### Commit Messages

Follow conventional commits:

```
type(scope): description

feat(terminal): add split pane support
fix(p2p): resolve connection timeout issue
docs(api): update composable documentation
refactor(stores): simplify connection state
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Pull Request Process

1. Create a feature branch from `main`
2. Make your changes
3. Run tests and linting
4. Submit PR with clear description
5. Address review feedback
6. Squash and merge when approved

## Code Standards

### TypeScript/Vue

```typescript
// Use composition API with <script setup>
<script setup lang="ts">
import { ref, computed } from 'vue';

// Props with types
const props = defineProps<{
  title: string;
  count?: number;
}>();

// Emits with types
const emit = defineEmits<{
  update: [value: string];
  close: [];
}>();

// Reactive state
const isOpen = ref(false);

// Computed properties
const displayTitle = computed(() => props.title.toUpperCase());
</script>
```

### Rust

```rust
// Use Result for fallible operations
pub async fn connect(&self, host: &str) -> Result<Session, Error> {
    // Implementation
}

// Document public APIs
/// Establishes an SSH connection to the specified host.
///
/// # Arguments
/// * `host` - The hostname or IP address
/// * `port` - The SSH port (default: 22)
///
/// # Returns
/// A `Session` on success, or an `Error` on failure.
pub async fn connect(&self, host: &str, port: u16) -> Result<Session, Error> {
    // Implementation
}
```

### CSS/Tailwind

- Use Tailwind utility classes
- Extract components for repeated patterns
- Use CSS variables for theming
- Support dark mode with `dark:` prefix

```vue
<template>
  <div class="p-4 bg-white dark:bg-gray-800 rounded-lg shadow-md">
    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
      {{ title }}
    </h2>
  </div>
</template>
```

## Testing

### Frontend Tests

```bash
# Run unit tests
pnpm test

# Run with coverage
pnpm test:coverage

# Run e2e tests
pnpm test:e2e
```

### Rust Tests

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p russh-ssh
```

### Writing Tests

```typescript
// Component test example
import { mount } from '@vue/test-utils';
import ConnectionCard from '@/components/connections/ConnectionCard.vue';

describe('ConnectionCard', () => {
  it('displays profile name', () => {
    const wrapper = mount(ConnectionCard, {
      props: {
        profile: {
          id: '1',
          name: 'Test Server',
          host: 'example.com',
          // ...
        }
      }
    });
    
    expect(wrapper.text()).toContain('Test Server');
  });
});
```

## Adding Features

### New Component

1. Create component in appropriate directory
2. Add TypeScript types if needed
3. Write unit tests
4. Update documentation
5. Add to relevant view/parent component

### New Composable

1. Create in `src/composables/`
2. Follow naming convention: `use[Name].ts`
3. Export from composable
4. Add cleanup in `onUnmounted`
5. Document in API.md

### New Store

1. Create in `src/stores/`
2. Use Pinia composition API
3. Export from `stores/index.ts`
4. Add TypeScript types
5. Document in API.md

### New Tauri Command

1. Add command in `src-tauri/src/`
2. Register in `main.rs`
3. Add TypeScript types
4. Create/update composable
5. Document in API.md

## Visual Effects

When adding visual effects:

1. Create component in `components/extra/`
2. Add settings to `types/visualEffects.ts`
3. Update `useVisualEffects` composable
4. Add toggle in VisualEffectsSettings
5. Respect `reducedMotion` preference
6. Clean up resources in `onUnmounted`

## Performance Guidelines

- Use `shallowRef` for large objects
- Implement virtual scrolling for long lists
- Lazy load routes and heavy components
- Clean up event listeners and timers
- Use `v-once` for static content
- Debounce expensive operations

## Accessibility

- Use semantic HTML elements
- Add ARIA labels where needed
- Support keyboard navigation
- Maintain focus management
- Test with screen readers
- Respect `prefers-reduced-motion`

## Documentation

- Update README for user-facing changes
- Update API.md for new APIs
- Add JSDoc comments to functions
- Include usage examples
- Keep CHANGELOG updated

## Getting Help

- Open an issue for bugs
- Use discussions for questions
- Join our Discord community
- Check existing issues first

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md
- Release notes
- Project README

Thank you for contributing! 🎉
