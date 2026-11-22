#!/bin/sh
# Installs FontCraft (macOS beta) into ~/Applications and opens it.
# Downloaded with curl, so macOS puts no quarantine flag on it: no Gatekeeper dialogs.
set -eu

REPO="mohseenjamall/FontCraft"
URL="https://github.com/$REPO/releases/download/mac-beta/FontCraft-macos.dmg"

[ "$(uname -s)" = "Darwin" ] || { echo "This installer is for macOS only." >&2; exit 1; }

TMP="$(mktemp -d)"
MNT="$TMP/mnt"
cleanup() { hdiutil detach "$MNT" -quiet 2>/dev/null || true; rm -rf "$TMP"; }
trap cleanup EXIT

echo "Downloading FontCraft..."
curl -fsSL "$URL" -o "$TMP/FontCraft.dmg"
mkdir -p "$MNT"
hdiutil attach "$TMP/FontCraft.dmg" -nobrowse -readonly -mountpoint "$MNT" -quiet

APP="$HOME/Applications/FontCraft.app"
pkill -x FontCraft 2>/dev/null || true
mkdir -p "$HOME/Applications"
rm -rf "$APP"
cp -R "$MNT/FontCraft.app" "$APP"
# Free ad-hoc signature (best effort); Apple Silicon wants every program signed.
codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || true

echo "Installed: $APP"
open "$APP"
