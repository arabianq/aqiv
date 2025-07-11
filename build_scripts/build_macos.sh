#!/usr/bin/bash

# Change dir to the cargo project
cd "$(dirname "$(realpath "$0")")/.." || exit

RUST_TARGET_AARCH64="aarch64-apple-darwin"
RUST_TARGET_X86="x86_64-apple-darwin"

cargo build --release --target "$RUST_TARGET_AARCH64"
cargo build --release --target "$RUST_TARGET_X86"

cargo install cargo-bundle
cargo bundle --release --target "$RUST_TARGET_AARCH64"
cargo bundle --release --target "$RUST_TARGET_X86"

fix_plist() {
  local app_path="$1"
  local plist="$app_path/Contents/Info.plist"

  echo "Patching $plist …"

  # 1) Delete old section if exists
  /usr/libexec/PlistBuddy -c "Delete :CFBundleDocumentTypes" "$plist" 2>/dev/null || true

  # 2) Create CFBundleDocumentTypes → array → dict
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes array" "$plist"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:0 dict" "$plist"

  # 3) Name and role
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:0:CFBundleTypeName string Image" "$plist"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:0:CFBundleTypeRole string Viewer" "$plist"

  # 4) Extensions
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:0:CFBundleTypeExtensions array" "$plist"
  for ext in avif bmp dds ff gif hdr ico jpeg exr png pnm qoi svg tga tiff webp; do
    /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:0:CFBundleTypeExtensions: string $ext" "$plist"
  done

  # 5) UTI
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:0:LSItemContentTypes array" "$plist"
  for uti in \
      public.heif \
      com.microsoft.bmp \
      com.microsoft.dds \
      public.image \
      public.gif \
      public.hdr-image \
      com.microsoft.ico \
      public.jpeg \
      public.png \
      public.pnm \
      public.image \
      public.svg-image \
      public.tga-image \
      public.tiff \
      public.webp; do
    /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:0:LSItemContentTypes: string $uti" "$plist"
  done

  echo "→ OK"
}

# Patch each bundle
fix_plist ./target/$RUST_TARGET_AARCH64/release/bundle/osx/AQIV.app
fix_plist ./target/$RUST_TARGET_X86/release/bundle/osx/AQIV.app
fix_plist ./target/universal_apple_darwin/bundle/osx/AQIV.app

# Compress binaries using upx (if upx is installed)
if command -v upx >/dev/null 2>&1; then
  # Compressed aarch64 macos binaries don't work =(
#  upx --force-macos --best ./target/$RUST_TARGET_AARCH64/release/aqiv
#  upx -t ./target/$RUST_TARGET_AARCH64/release/aqiv

  upx --force-macos --best ./target/$RUST_TARGET_X86/release/aqiv
  upx -t ./target/$RUST_TARGET_X86/release/aqiv
  cp ./target/$RUST_TARGET_X86/release/aqiv ./target/$RUST_TARGET_X86/bundle/osx/AQIV.app/Contents/MacOS/aqiv
fi

# Create universal binary
mkdir ./target/universal_apple_darwin
lipo -create -output target/universal_apple_darwin/aqiv ./target/$RUST_TARGET_AARCH64/release/aqiv ./target/$RUST_TARGET_X86/release/aqiv
cp -r ./target/$RUST_TARGET_AARCH64/release/bundle ./target/universal_apple_darwin/
cp ./target/universal_apple_darwin/aqiv ./target/universal_apple_darwin/bundle/osx/AQIV.app/Contents/MacOS/aqiv

# Create .dmg installers for all .app files
create-dmg --app-drop-link 20 20 aqiv-macos-aarch64.dmg ./target/$RUST_TARGET_AARCH64/release/bundle/osx/AQIV.app
create-dmg --app-drop-link 20 20 aqiv-macos-x86_64.dmg ./target/$RUST_TARGET_X86/release/bundle/osx/AQIV.app
create-dmg --app-drop-link 20 20 aqiv-macos-universal.dmg ./target/universal_apple_darwin/bundle/osx/AQIV.app

# Copy all .dmg file into final directory
mkdir ./target/macos-for-github-release
mv aqiv-macos-aarch64.dmg ./target/macos-for-github-release/
mv aqiv-macos-x86_64.dmg ./target/macos-for-github-release/
mv aqiv-macos-universal.dmg ./target/macos-for-github-release/