# Building Unsigned iOS IPA

This guide explains how to create an unsigned `.ipa` file for iOS distribution (sideloading, enterprise, or TestFlight).

## Prerequisites

- macOS with Xcode installed
- Tauri CLI (`npm install -g @tauri-apps/cli`)
- Rust with iOS targets: `rustup target add aarch64-apple-ios`

## Quick Build (Automated)

```bash
cd russh-client

# Initialize iOS project (first time only)
npm run tauri ios init

# Build release
npm run tauri ios build --release
```

## Manual IPA Creation

### Step 1: List Available Schemes

```bash
cd src-tauri/gen/apple

# For .xcodeproj
xcodebuild -list -project *.xcodeproj

# For .xcworkspace
xcodebuild -list -workspace *.xcworkspace
```

### Step 2: Build Unsigned Archive

```bash
# Using .xcodeproj
xcodebuild archive \
  -project RUSSH_iOS.xcodeproj \
  -scheme "RUSSH_iOS" \
  -archivePath unsigned.xcarchive \
  -configuration Release \
  -destination "generic/platform=iOS" \
  CODE_SIGN_IDENTITY="" \
  CODE_SIGNING_REQUIRED=NO \
  CODE_SIGNING_ALLOWED=NO \
  DEVELOPMENT_TEAM=""

# Using .xcworkspace (if present)
xcodebuild archive \
  -workspace RUSSH.xcworkspace \
  -scheme "RUSSH_iOS" \
  -archivePath unsigned.xcarchive \
  -configuration Release \
  -destination "generic/platform=iOS" \
  CODE_SIGN_IDENTITY="" \
  CODE_SIGNING_REQUIRED=NO \
  CODE_SIGNING_ALLOWED=NO \
  DEVELOPMENT_TEAM=""
```

### Step 3: Create IPA from Archive

```bash
# Navigate to archive
cd unsigned.xcarchive/Products

# Rename Applications to Payload
mv Applications Payload

# Create IPA
zip -r RUSSH-unsigned.ipa Payload

# Move to project root
mv RUSSH-unsigned.ipa ../../../
```

## One-Liner Script

```bash
#!/bin/bash
cd src-tauri/gen/apple

SCHEME=$(xcodebuild -list -project *.xcodeproj 2>/dev/null | grep -A 100 "Schemes:" | grep -v "Schemes:" | head -1 | xargs)

xcodebuild archive \
  -project *.xcodeproj \
  -scheme "$SCHEME" \
  -archivePath build/unsigned.xcarchive \
  -configuration Release \
  -destination "generic/platform=iOS" \
  CODE_SIGN_IDENTITY="" \
  CODE_SIGNING_REQUIRED=NO \
  CODE_SIGNING_ALLOWED=NO

cd build/unsigned.xcarchive/Products
mv Applications Payload
zip -r ../../../RUSSH-unsigned.ipa Payload
echo "✅ Created: RUSSH-unsigned.ipa"
```

## CI/CD

Use the GitHub Actions workflow:

1. Go to **Actions** → **Mobile Build**
2. Select **ios** platform
3. Choose **release** build type
4. Click **Run workflow**

The unsigned IPA will be available as a build artifact.

## Installing Unsigned IPA

### Using AltStore/Sideloadly
1. Download the unsigned IPA
2. Use AltStore or Sideloadly to install on your device
3. Trust the developer certificate in Settings → General → Device Management

### Using Xcode
1. Open Xcode → Window → Devices and Simulators
2. Select your device
3. Drag the IPA to the "Installed Apps" section

## Troubleshooting

### "No scheme found"
```bash
# List all schemes
xcodebuild -list -project *.xcodeproj
```

### "Code signing required"
Ensure all CODE_SIGN flags are set:
```bash
CODE_SIGN_IDENTITY=""
CODE_SIGNING_REQUIRED=NO
CODE_SIGNING_ALLOWED=NO
DEVELOPMENT_TEAM=""
```

### Build fails with missing dependencies
```bash
# Reinstall iOS targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

# Reinitialize project
rm -rf src-tauri/gen/apple
npm run tauri ios init
```
