#!/usr/bin/env bash
# Build a premium, custom-background macOS installer DMG for FerrumGrid.
# Uses native macOS tools (hdiutil + osascript) to achieve flawless, high-end design styling.

set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: macOS-only script (uses hdiutil + osascript)" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-0.3.9}"
TARGET="${2:-aarch64-apple-darwin}"
APP="$ROOT/FerrumGrid.app"
OUT_DMG="$ROOT/FerrumGrid-${VERSION}-${TARGET}.dmg"

if [[ ! -d "$APP" ]]; then
  echo "error: FerrumGrid.app bundle not found at $APP" >&2
  exit 1
fi

BG_IMG="$ROOT/assets/dmg-background.png"
if [[ ! -f "$BG_IMG" ]]; then
  echo "error: DMG background image not found at $BG_IMG" >&2
  exit 1
fi

WORK="$(mktemp -d)"
TEMP_DMG="$WORK/temp.dmg"
STAGING="$WORK/staging"
MOUNT_DIR="$WORK/mnt"

mkdir -p "$STAGING" "$MOUNT_DIR"

trap 'rm -rf "$WORK"' EXIT

echo "Preparing staging area..."
# Copy App and create Applications symlink
cp -R "$APP" "$STAGING/FerrumGrid.app"
ln -s /Applications "$STAGING/Applications"
# Copy background image directly and hide it graphically using macOS system chflags
cp "$BG_IMG" "$STAGING/background.png"
chflags hidden "$STAGING/background.png"

# Ensure all staging files are fully write-enabled to prevent Finder coordinate write locks
chmod -R u+w "$STAGING"

# We calculate size based on app size + background + buffer
APP_SIZE=$(du -sk "$APP" | cut -f1)
DMG_SIZE=$(( (APP_SIZE + 10240) / 1024 )) # size in MB

echo "Creating temporary read-write DMG (size: ${DMG_SIZE}MB)..."
hdiutil create -volname "FerrumGrid" -srcfolder "$STAGING" -size "${DMG_SIZE}m" -fs HFS+ -ov -format UDRW "$TEMP_DMG"

echo "Mounting temporary DMG..."
MOUNT_OUTPUT=$(hdiutil attach -readwrite -noverify -noautoopen "$TEMP_DMG")
MOUNT_DIR=$(echo "$MOUNT_OUTPUT" | grep -o '/Volumes/[^/]*' | head -n 1 | xargs)
VOL_NAME=$(basename "$MOUNT_DIR")

echo "Mounted at: $MOUNT_DIR (Volume Name: $VOL_NAME)"

# Wait a brief moment to ensure macOS Finder is ready
sleep 2

echo "Applying premium Finder styling and layout positions via AppleScript..."
osascript -e "
  tell application \"Finder\"
    activate
    tell disk \"$VOL_NAME\" to open
    delay 3
    set the_window to window 1
    
    set current view of the_window to icon view
    set toolbar visible of the_window to false
    set statusbar visible of the_window to false
    
    -- Set perfect window dimensions and center it (bounds: {left, top, right, bottom})
    -- 640x400 window aspect ratio
    set bounds of the_window to {200, 200, 840, 600}
    
    -- Wait for Finder to transition the view mode before setting icon properties
    delay 1
    
    try
      set the_options to icon view options of the_window
      set icon size of the_options to 104
      set arrangement of the_options to not arranged
    on error
      -- Ignore icon size write failures on restricted locales/versions
    end try
    
    try
      set the_options to icon view options of the_window
      set background picture of the_options to file \"background.png\" of disk \"$VOL_NAME\"
    on error
      try
        -- Double-fallback using POSIX file syntax
        set background picture of the_options to POSIX file \"$MOUNT_DIR/background.png\"
      on error
        -- Fallback if background picture setting has an alternate path
      end try
    end try
    
    -- Tell the disk itself to position the items (Finder's native container for files)
    tell disk \"$VOL_NAME\"
      try
        set position of item \"FerrumGrid\" to {160, 240}
      on error
        try
          set position of item \"FerrumGrid.app\" to {160, 240}
        on error
        end try
      end try
      
      try
        set position of item \"Applications\" to {480, 240}
      on error
      end try
    end tell
    
    delay 1
    close the_window
  end tell
"

echo "Blessing and unmounting DMG volume..."
# Tell diskutil to sync and detach
diskutil eject "$MOUNT_DIR" || hdiutil detach "$MOUNT_DIR" -force

echo "Compressing read-write DMG into final premium UDZO format..."
rm -f "$OUT_DMG"
hdiutil convert "$TEMP_DMG" -format UDZO -imagekey zlib-level=9 -o "$OUT_DMG"

echo "Wrote final premium DMG to $OUT_DMG successfully!"
