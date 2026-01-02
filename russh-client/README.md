# RUSSH Client

The frontend application for RUSSH, built with Vue 3, TypeScript, and Tauri.

## Development

```bash
# Install dependencies
pnpm install

# Start development server
pnpm tauri dev

# Build for production
pnpm tauri build

# Run tests
pnpm test

# Lint code
pnpm lint
```

## Project Structure

```
src/
├── assets/          # Static assets and styles
├── components/      # Vue components
│   ├── common/      # Shared components
│   ├── connections/ # Connection management
│   ├── extra/       # Visual effect components
│   ├── files/       # File browser components
│   ├── mobile/      # Mobile-specific components
│   ├── p2p/         # P2P networking components
│   ├── settings/    # Settings components
│   └── terminal/    # Terminal components
├── composables/     # Vue composables
├── router/          # Vue Router configuration
├── stores/          # Pinia stores
├── types/           # TypeScript type definitions
├── utils/           # Utility functions
└── views/           # Page views
```

## Key Dependencies

| Package | Purpose |
|---------|---------|
| vue | UI framework |
| pinia | State management |
| vue-router | Routing |
| @tauri-apps/api | Tauri IPC |
| xterm | Terminal emulator |
| lucide-vue-next | Icons |
| motion-v | Animations |
| tailwindcss | Styling |

## Configuration

### Tailwind CSS

Configuration in `tailwind.config.js`:
- Dark mode support
- Custom color palette
- Extended animations

### TypeScript

Configuration in `tsconfig.json`:
- Strict mode enabled
- Path aliases (`@/` → `src/`)
- Vue SFC support

### Vite

Configuration in `vite.config.ts`:
- Vue plugin
- Path aliases
- Development server settings

## Scripts

| Script | Description |
|--------|-------------|
| `pnpm dev` | Start Vite dev server |
| `pnpm build` | Build for production |
| `pnpm preview` | Preview production build |
| `pnpm tauri dev` | Start Tauri development |
| `pnpm tauri build` | Build Tauri application |
| `pnpm test` | Run unit tests |
| `pnpm lint` | Run ESLint |
| `pnpm type-check` | Run TypeScript check |

## Environment Variables

Create `.env.local` for local development:

```env
VITE_APP_TITLE=RUSSH
VITE_API_URL=http://localhost:3000
```

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

WebGL required for visual effects.
