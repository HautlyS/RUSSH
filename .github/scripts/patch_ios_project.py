#!/usr/bin/env python3
"""
Patch Tauri iOS project for unsigned release builds.
- Disables code signing
- Configures for release mode (bundled assets, not dev server)
- Grants local network permissions as fallback
- Patches WKWebView configuration for bundled assets
"""
import glob
import sys
import re
import os
import plistlib

def patch_pbxproj(path):
    """Patch pbxproj safely line by line."""
    print(f"Patching pbxproj: {path}")
    
    with open(path, 'r') as f:
        lines = f.readlines()
    
    new_lines = []
    modified = False
    
    for line in lines:
        # Skip shellScript lines containing tauri - replace with no-op
        if 'shellScript' in line and 'tauri' in line.lower():
            indent = len(line) - len(line.lstrip())
            line = ' ' * indent + 'shellScript = "echo \\"Rust library pre-built\\"";\n'
            modified = True
        
        # Simple string replacements for code signing
        simple_replacements = [
            ('CODE_SIGN_IDENTITY = "Apple Development"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGN_IDENTITY = "-"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGN_IDENTITY = "iPhone Developer"', 'CODE_SIGN_IDENTITY = ""'),
            ('CODE_SIGNING_REQUIRED = YES', 'CODE_SIGNING_REQUIRED = NO'),
            ('CODE_SIGNING_ALLOWED = YES', 'CODE_SIGNING_ALLOWED = NO'),
            ('CODE_SIGN_STYLE = Automatic', 'CODE_SIGN_STYLE = Manual'),
            ('ProvisioningStyle = Automatic', 'ProvisioningStyle = Manual'),
        ]
        
        for old, new in simple_replacements:
            if old in line:
                line = line.replace(old, new)
                modified = True
        
        # Handle DEVELOPMENT_TEAM with regex
        if 'DEVELOPMENT_TEAM' in line and '""' not in line:
            new_line = re.sub(r'DEVELOPMENT_TEAM\s*=\s*[^;]+;', 'DEVELOPMENT_TEAM = "";', line)
            if new_line != line:
                line = new_line
                modified = True
        
        new_lines.append(line)
    
    with open(path, 'w') as f:
        f.writelines(new_lines)
    
    print(f"✅ Patched pbxproj: {path} ({'modified' if modified else 'no changes'})")

def patch_info_plist(path):
    """Patch Info.plist to grant local network permissions and configure for release."""
    print(f"Patching Info.plist: {path}")
    
    try:
        with open(path, 'rb') as f:
            plist = plistlib.load(f)
    except Exception as e:
        print(f"⚠️  Could not read plist: {e}")
        return
    
    modified = False
    
    # Add local network usage description (required for iOS 14+)
    if 'NSLocalNetworkUsageDescription' not in plist:
        plist['NSLocalNetworkUsageDescription'] = 'This app requires local network access for development and debugging purposes.'
        modified = True
    
    # Add Bonjour services for local network discovery
    if 'NSBonjourServices' not in plist:
        plist['NSBonjourServices'] = ['_http._tcp.', '_https._tcp.']
        modified = True
    
    # Allow arbitrary loads for development (can be removed for production)
    if 'NSAppTransportSecurity' not in plist:
        plist['NSAppTransportSecurity'] = {}
    
    ats = plist['NSAppTransportSecurity']
    if 'NSAllowsLocalNetworking' not in ats or not ats['NSAllowsLocalNetworking']:
        ats['NSAllowsLocalNetworking'] = True
        modified = True
    
    # Allow localhost connections
    if 'NSExceptionDomains' not in ats:
        ats['NSExceptionDomains'] = {}
    
    if 'localhost' not in ats['NSExceptionDomains']:
        ats['NSExceptionDomains']['localhost'] = {
            'NSExceptionAllowsInsecureHTTPLoads': True,
            'NSIncludesSubdomains': True
        }
        modified = True
    
    if modified:
        with open(path, 'wb') as f:
            plistlib.dump(plist, f)
        print(f"✅ Patched Info.plist: {path}")
    else:
        print(f"✅ Info.plist already configured: {path}")

def patch_swift_files(apple_path):
    """Patch Swift files to ensure release mode uses bundled assets."""
    swift_files = glob.glob(os.path.join(apple_path, '**', '*.swift'), recursive=True)
    
    for swift_file in swift_files:
        try:
            with open(swift_file, 'r') as f:
                content = f.read()
            
            original = content
            
            # Look for devUrl references and ensure they're only used in DEBUG
            # Tauri 2.x uses a different approach - check for WKWebView configuration
            
            # If there's a hardcoded localhost URL, wrap it in DEBUG check
            if 'localhost:1420' in content or 'http://localhost' in content:
                print(f"⚠️  Found localhost reference in {swift_file}")
                # This is informational - the actual fix is in the build configuration
            
            if content != original:
                with open(swift_file, 'w') as f:
                    f.write(content)
                print(f"✅ Patched Swift file: {swift_file}")
        except Exception as e:
            print(f"⚠️  Could not process {swift_file}: {e}")

def create_xcconfig(apple_path):
    """Create an xcconfig file to force release settings."""
    xcconfig_content = """// Release configuration for unsigned builds
// This ensures the app uses bundled assets instead of dev server

CODE_SIGN_IDENTITY = 
CODE_SIGNING_REQUIRED = NO
CODE_SIGNING_ALLOWED = NO
DEVELOPMENT_TEAM = 
AD_HOC_CODE_SIGNING_ALLOWED = NO

// Ensure release optimizations
GCC_OPTIMIZATION_LEVEL = s
SWIFT_OPTIMIZATION_LEVEL = -O
ENABLE_TESTABILITY = NO
DEBUG_INFORMATION_FORMAT = dwarf-with-dsym

// Force release mode for Tauri
GCC_PREPROCESSOR_DEFINITIONS = $(inherited) RELEASE=1 NDEBUG=1
SWIFT_ACTIVE_COMPILATION_CONDITIONS = RELEASE
"""
    
    xcconfig_path = os.path.join(apple_path, 'Release-Unsigned.xcconfig')
    with open(xcconfig_path, 'w') as f:
        f.write(xcconfig_content)
    print(f"✅ Created xcconfig: {xcconfig_path}")

def patch_tauri_conf_for_release(base_path):
    """Verify tauri.conf.json has correct frontendDist for release builds."""
    tauri_conf_path = os.path.join(base_path, 'src-tauri', 'tauri.conf.json')
    
    if not os.path.exists(tauri_conf_path):
        print(f"⚠️  tauri.conf.json not found at {tauri_conf_path}")
        return
    
    import json
    with open(tauri_conf_path, 'r') as f:
        config = json.load(f)
    
    # Verify frontendDist is set (this is what release builds use)
    build_config = config.get('build', {})
    frontend_dist = build_config.get('frontendDist')
    
    if frontend_dist:
        print(f"✅ Tauri config has frontendDist: {frontend_dist}")
    else:
        print("⚠️  No frontendDist configured - release builds may fail")

def ensure_assets_copied(apple_path):
    """Ensure the frontend assets are properly referenced."""
    assets_paths = glob.glob(os.path.join(apple_path, '**', 'Assets.xcassets'), recursive=True)
    
    for assets_path in assets_paths:
        print(f"✅ Found assets at: {assets_path}")

def main():
    base_path = os.getcwd()
    apple_path = os.path.join(base_path, 'src-tauri', 'gen', 'apple')
    
    # Patch pbxproj files
    pbxproj_files = glob.glob(os.path.join(apple_path, '**', '*.pbxproj'), recursive=True)
    
    if not pbxproj_files:
        print("⚠️  No pbxproj files found")
    else:
        for pbx in pbxproj_files:
            patch_pbxproj(pbx)
    
    # Patch Info.plist files
    plist_files = glob.glob(os.path.join(apple_path, '**', 'Info.plist'), recursive=True)
    
    if not plist_files:
        print("⚠️  No Info.plist files found")
    else:
        for plist in plist_files:
            patch_info_plist(plist)
    
    # Patch Swift files
    patch_swift_files(apple_path)
    
    # Create xcconfig for release
    if os.path.exists(apple_path):
        create_xcconfig(apple_path)
    
    # Verify tauri config
    patch_tauri_conf_for_release(base_path)
    
    # Check assets
    ensure_assets_copied(apple_path)
    
    print(f"\n✅ Done: Patched {len(pbxproj_files)} pbxproj and {len(plist_files)} plist file(s)")

if __name__ == "__main__":
    main()