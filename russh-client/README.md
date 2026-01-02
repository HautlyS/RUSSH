# RUSSH Client

A cross-platform SSH client built with Tauri 2.0 and Vue.js 3.

## Features

- 🔐 Secure SSH connections with password and key-based authentication
- 📁 SFTP file browser with drag-and-drop support
- 🖥️ Full-featured terminal with xterm.js
- 🌐 P2P device connectivity
- 📱 Mobile support (iOS & Android)
- 🎨 Customizable themes and visual effects
- ♿ Accessibility compliant (WCAG 2.1 AA)

## Development

### Prerequisites

- Node.js 20+
- Rust 1.70+
- Platform-specific dependencies (see [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites))

### Setup

```bash
cd russh-client
npm install
npm run tauri dev
```

### Building

```bash
# Desktop (current platform)
npm run tauri build

# Desktop (specific target)
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target aarch64-apple-darwin
npm run tauri build -- --target x86_64-pc-windows-msvc
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

## Mobile Builds

### iOS (Unsigned IPA)

```bash
# Initialize iOS project
npm run tauri ios init

# Build
npm run tauri ios build --release

# Create unsigned IPA manually:
cd src-tauri/gen/apple

# 1. List schemes
xcodebuild -list -project *.xcodeproj

# 2. Build xcarchive (unsigned)
xcodebuild archive \
  -project *.xcodeproj \
  -scheme "RUSSH_iOS" \
  -archivePath unsigned.xcarchive \
  -configuration Release \
  -destination "generic/platform=iOS" \
  CODE_SIGN_IDENTITY="" \
  CODE_SIGNING_REQUIRED=NO \
  CODE_SIGNING_ALLOWED=NO

# 3. Create IPA
cd unsigned.xcarchive/Products
mv Applications Payload
zip -r RUSSH-unsigned.ipa Payload
```

### Android (APK)

```bash
# Initialize Android project
npm run tauri android init

# Build debug APK
npm run tauri android build --apk --debug

# Build release APK
npm run tauri android build --apk

# APK location: src-tauri/gen/android/app/build/outputs/apk/
```

## Testing

```bash
npm run test:unit      # Unit tests
npm run test:e2e       # E2E tests (Playwright)
npm run test:coverage  # Coverage report
```

## CI/CD

The project uses GitHub Actions for automated builds:

- **CI** (`ci.yml`): Runs on push/PR, builds all platforms
- **Release** (`release.yml`): Creates releases on version tags
- **Mobile** (`mobile.yml`): Manual trigger for iOS/Android builds

### Creating a Release

```bash
git tag v0.1.0
git push origin v0.1.0
```

### Manual Mobile Build

Go to Actions → Mobile Build → Run workflow

## Project Structure

```
russh-client/
├── src/                    # Vue.js frontend
│   ├── components/         # Vue components
│   ├── composables/        # Vue composables
│   ├── stores/             # Pinia stores
│   └── views/              # Page views
├── src-tauri/              # Tauri backend
│   ├── src/commands/       # Tauri commands
│   ├── gen/apple/          # iOS config
│   └── gen/android/        # Android config
└── package.json
```

## License

MIT
