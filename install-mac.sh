#!/bin/sh
# Installs FontCraft (macOS beta) as ~/Applications/FontCraft.app and opens it.
# Downloaded with curl, so macOS puts no quarantine flag on it: no Gatekeeper dialogs.
set -eu

REPO="mohseenjamall/FontCraft"
BASE="https://github.com/$REPO/releases/download/mac-beta"
ICON_URL="https://raw.githubusercontent.com/$REPO/mac-linux-support/assets/app-art.png"

[ "$(uname -s)" = "Darwin" ] || { echo "This installer is for macOS only." >&2; exit 1; }
case "$(uname -m)" in
  arm64) NAME="FontCraft-macos-arm64" ;;
  x86_64) NAME="FontCraft-macos-x64" ;;
  *) echo "Unsupported Mac processor: $(uname -m)" >&2; exit 1 ;;
esac

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading FontCraft..."
curl -fsSL "$BASE/$NAME.tar.gz" -o "$TMP/app.tar.gz"
tar -xzf "$TMP/app.tar.gz" -C "$TMP"

APP="$HOME/Applications/FontCraft.app"
pkill -x FontCraft 2>/dev/null || true
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$TMP/$NAME" "$APP/Contents/MacOS/FontCraft"
chmod +x "$APP/Contents/MacOS/FontCraft"

# Icon (optional: the app works without it).
if curl -fsSL "$ICON_URL" -o "$TMP/icon.png" 2>/dev/null; then
  SET="$TMP/AppIcon.iconset"
  mkdir -p "$SET"
  for s in 16 32 64 128 256; do
    sips -z "$s" "$s" "$TMP/icon.png" --out "$SET/icon_${s}x${s}.png" >/dev/null 2>&1 || true
  done
  iconutil -c icns "$SET" -o "$APP/Contents/Resources/AppIcon.icns" >/dev/null 2>&1 || true
fi

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>FontCraft</string>
  <key>CFBundleDisplayName</key><string>FontCraft</string>
  <key>CFBundleIdentifier</key><string>com.fontcraft.app</string>
  <key>CFBundleExecutable</key><string>FontCraft</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>1.1.0</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Free ad-hoc signature (best effort); Apple Silicon wants every program signed.
codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || true

echo "Installed: $APP"
open "$APP"
