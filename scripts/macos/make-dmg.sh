#!/usr/bin/env bash
# Packs FontCraft.app into a drag-to-install disk image (the app next to a link to /Applications).
# Usage: make-dmg.sh <FontCraft.app> <out.dmg>
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <FontCraft.app> <out.dmg>" >&2
  exit 2
fi
app=$1 dmg=$2

stage=$(mktemp -d)
cp -R "$app" "$stage/"
ln -s /Applications "$stage/Applications"
rm -f "$dmg"
hdiutil create -volname FontCraft -srcfolder "$stage" -ov -format UDZO "$dmg"
rm -rf "$stage"

echo "built $dmg"
