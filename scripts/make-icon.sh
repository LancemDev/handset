#!/usr/bin/env bash
# Rasterise the icon SVGs into app/packaging/AppIcon.icns (macOS). Run: scripts/make-icon.sh
# Needs rsvg-convert (brew install librsvg) and iconutil. Sizes up to 64 px use the simplified
# small drawing, larger sizes use the full one.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
src="$root/app/packaging/icon"
set_dir="$(mktemp -d)/AppIcon.iconset"
mkdir -p "$set_dir"

render() { # <px> <svg> <out>
  rsvg-convert -w "$1" -h "$1" "$2" -o "$3"
}
pick() { if [ "$1" -le 64 ]; then echo "$src/handset-icon-small.svg"; else echo "$src/handset-icon.svg"; fi; }

for base in 16 32 128 256 512; do
  render "$base" "$(pick "$base")" "$set_dir/icon_${base}x${base}.png"
  render "$((base * 2))" "$(pick "$((base * 2))")" "$set_dir/icon_${base}x${base}@2x.png"
done

iconutil -c icns "$set_dir" -o "$root/app/packaging/AppIcon.icns"
render 512 "$src/handset-icon.svg" "$src/handset-icon-512.png"
echo "wrote app/packaging/AppIcon.icns and app/packaging/icon/handset-icon-512.png"
