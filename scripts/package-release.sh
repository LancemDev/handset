#!/usr/bin/env bash
# Build the release files in dist/release/ (macOS): the universal app zip, the command line tool
# archive and SHA256SUMS. Run: scripts/package-release.sh
#
# Signing and notarising (optional, see docs/runbooks/release.md):
#   CODESIGN_IDENTITY   Developer ID Application identity or certificate hash
#   NOTARY_PROFILE      notarytool keychain profile, or
#   NOTARY_KEY_PATH, NOTARY_KEY_ID, NOTARY_ISSUER   App Store Connect API key
# With an identity the app and the command line tool are signed with the hardened runtime; with
# notary credentials too they are notarised and the app is stapled. Without either the build is ad hoc.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(grep -m1 '^version' "$root/app/Cargo.toml" | cut -d'"' -f2)"
out="$root/dist/release"

UNIVERSAL=1 "$root/scripts/package-macos.sh"

rm -rf "$out"
mkdir -p "$out"

identity="${CODESIGN_IDENTITY:--}"
cli_bin="$out/handset"
cp "$root/dist/Handset.app/Contents/MacOS/handset" "$cli_bin"
if [ "$identity" != "-" ]; then
  codesign --force --options runtime --timestamp --sign "$identity" "$cli_bin"
fi

notary=()
if [ -n "${NOTARY_PROFILE:-}" ]; then
  notary=(--keychain-profile "$NOTARY_PROFILE")
elif [ -n "${NOTARY_KEY_PATH:-}" ]; then
  notary=(--key "$NOTARY_KEY_PATH" --key-id "$NOTARY_KEY_ID" --issuer "$NOTARY_ISSUER")
fi
if [ "$identity" != "-" ] && [ ${#notary[@]} -gt 0 ]; then
  # one submission covers the app and the command line tool
  stage="$(mktemp -d)"
  cp -R "$root/dist/Handset.app" "$cli_bin" "$stage/"
  ditto -c -k --keepParent "$stage" "$out/notarize-submission.zip"
  xcrun notarytool submit "$out/notarize-submission.zip" "${notary[@]}" --wait
  xcrun stapler staple "$root/dist/Handset.app"
  rm -rf "$stage" "$out/notarize-submission.zip"
elif [ "$identity" != "-" ]; then
  echo "signed but not notarised (no notary credentials): Gatekeeper will still warn" >&2
fi

# ditto keeps the code signature and extended attributes that plain zip drops
ditto -c -k --keepParent "$root/dist/Handset.app" "$out/Handset-$version-macos.zip"

cli="$out/handset-$version-macos"
mkdir -p "$cli"
mv "$cli_bin" "$cli/handset"
cp "$root/LICENSE" "$root/NOTICE" "$cli/"
tar -C "$out" -czf "$out/handset-$version-macos.tar.gz" "handset-$version-macos"
rm -rf "$cli"

(cd "$out" && shasum -a 256 -- *.zip *.tar.gz > SHA256SUMS)
echo "built release files for $version:"
ls -la "$out"
lipo -archs "$root/dist/Handset.app/Contents/MacOS/handset"
