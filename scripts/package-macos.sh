#!/usr/bin/env bash
# Build dist/Handset.app (macOS). Run: scripts/package-macos.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
app="$root/dist/Handset.app"
version="$(grep -m1 '^version' "$root/app/Cargo.toml" | cut -d'"' -f2)"

target="${CARGO_TARGET_DIR:-$root/app/target}"
(cd "$root/app" && cargo build --release)

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$target/release/handset" "$app/Contents/MacOS/handset"
cp "$root/app/packaging/AppIcon.icns" "$app/Contents/Resources/AppIcon.icns"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Handset</string>
  <key>CFBundleDisplayName</key><string>Handset</string>
  <key>CFBundleIdentifier</key><string>com.davidamunga.handset</string>
  <key>CFBundleExecutable</key><string>handset</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict>
</plist>
PLIST

# ad-hoc signature: enough to run on this Mac (not for distribution)
codesign --force --deep --sign - "$app"
plutil -lint "$app/Contents/Info.plist" >/dev/null
echo "built $app"
