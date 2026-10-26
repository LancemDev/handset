#!/usr/bin/env bash
# Build the release files in dist/release/ (macOS): the universal app zip, the command line tool
# archive and SHA256SUMS. Run: scripts/package-release.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(grep -m1 '^version' "$root/app/Cargo.toml" | cut -d'"' -f2)"
out="$root/dist/release"

UNIVERSAL=1 "$root/scripts/package-macos.sh"

rm -rf "$out"
mkdir -p "$out"

# ditto keeps the code signature and extended attributes that plain zip drops
ditto -c -k --keepParent "$root/dist/Handset.app" "$out/Handset-$version-macos.zip"

cli="$out/handset-$version-macos"
mkdir -p "$cli"
cp "$root/dist/Handset.app/Contents/MacOS/handset" "$cli/handset"
cp "$root/LICENSE" "$root/NOTICE" "$cli/"
tar -C "$out" -czf "$out/handset-$version-macos.tar.gz" "handset-$version-macos"
rm -rf "$cli"

(cd "$out" && shasum -a 256 -- *.zip *.tar.gz > SHA256SUMS)
echo "built release files for $version:"
ls -la "$out"
lipo -archs "$root/dist/Handset.app/Contents/MacOS/handset"
