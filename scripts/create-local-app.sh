#!/usr/bin/env bash
# Assemble a local FerrumGrid.app bundle from the compiled target/release/ferrumgrid binary.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/ferrumgrid"

if [[ ! -f "$BIN" ]]; then
  echo "error: release binary not found at $BIN. Run 'cargo build --release' first." >&2
  exit 1
fi

APP="$ROOT/FerrumGrid.app"
echo "Cleaning and building $APP..."
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp "$BIN" "$APP/Contents/MacOS/ferrumgrid"
chmod +x "$APP/Contents/MacOS/ferrumgrid"
cp "$ROOT/assets/AppIcon.icns" "$APP/Contents/Resources/AppIcon.icns"
echo -n "APPL????" > "$APP/Contents/PkgInfo"


cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>ferrumgrid</string>
  <key>CFBundleIdentifier</key><string>com.stormix.ferrumgrid</string>
  <key>CFBundleName</key><string>FerrumGrid</string>
  <key>CFBundleDisplayName</key><string>FerrumGrid</string>
  <key>CFBundleVersion</key><string>0.3.8</string>
  <key>CFBundleShortVersionString</key><string>0.3.8</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleSignature</key><string>????</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
EOF

echo "Successfully assembled $APP!"
