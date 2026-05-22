#!/usr/bin/env bash
# Render assets/app-icon-dark.svg → assets/AppIcon.icns for the macOS .app bundle.
# Run locally on macOS when the source SVG changes; commit the resulting .icns.
#
# Why local-only: qlmanage's SVG QuickLook generator isn't reliably present on
# CI runners, and we don't want to depend on Homebrew librsvg there. The .icns
# is small (~few hundred KB) and rarely changes, so we ship it as a tracked
# artifact instead of regenerating per-build.

set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: macOS-only (uses qlmanage + sips + iconutil)" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/assets/app-icon.png"
OUT="$ROOT/assets/AppIcon.icns"

if [[ ! -f "$SRC" ]]; then
  echo "error: source PNG not found at $SRC" >&2
  exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

MASTER="$SRC"

# 2) Build .iconset with all sizes Apple expects.
SET="$WORK/AppIcon.iconset"
mkdir -p "$SET"

scale() {
  # $1 = pixel size, $2 = filename inside iconset
  sips -z "$1" "$1" "$MASTER" --out "$SET/$2" >/dev/null
}

scale   16 icon_16x16.png
scale   32 icon_16x16@2x.png
scale   32 icon_32x32.png
scale   64 icon_32x32@2x.png
scale  128 icon_128x128.png
scale  256 icon_128x128@2x.png
scale  256 icon_256x256.png
scale  512 icon_256x256@2x.png
scale  512 icon_512x512.png
scale 1024 icon_512x512@2x.png

# 3) iconset → .icns
iconutil -c icns -o "$OUT" "$SET"
echo "wrote $OUT"
