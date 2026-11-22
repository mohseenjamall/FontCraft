#!/usr/bin/env bash
# Builds FontCraft.app from a (universal) binary. macOS only (sips, iconutil).
# Usage: bundle.sh <version> <universal-binary> <out-dir>   -> <out-dir>/FontCraft.app
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: $0 <version> <universal-binary> <out-dir>" >&2
  exit 2
fi
version=$1 binary=$2 out=$3
here=$(cd "$(dirname "$0")" && pwd)
art="$here/../../assets/app-art.png"
app="$out/FontCraft.app"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$binary" "$app/Contents/MacOS/fontcraft"
chmod 755 "$app/Contents/MacOS/fontcraft"
sed "s/@VERSION@/$version/g" "$here/Info.plist" > "$app/Contents/Info.plist"

# The art is 256x256; the larger iconset sizes are upscaled.
iconset=$(mktemp -d)/AppIcon.iconset
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$art" --out "$iconset/icon_${size}x${size}.png" > /dev/null
  sips -z $((size * 2)) $((size * 2)) "$art" --out "$iconset/icon_${size}x${size}@2x.png" > /dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/AppIcon.icns"
rm -rf "$(dirname "$iconset")"

echo "built $app"
