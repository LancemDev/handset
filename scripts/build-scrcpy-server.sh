#!/usr/bin/env bash
# Rebuild app/assets/scrcpy-server from the scrcpy source at a given tag (default: the pinned one).
# Needs a JDK (17) and the Android SDK (ANDROID_HOME). See docs/runbooks/rebuild-scrcpy-server.md.
set -euo pipefail

tag="${1:-v5.0.1}"
root="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

git clone --depth 1 --branch "$tag" https://github.com/Genymobile/scrcpy.git "$work/scrcpy"
(cd "$work/scrcpy" && ./gradlew --no-daemon -q assembleDebug)

cp "$work/scrcpy/server/build/outputs/apk/debug/server-debug.apk" "$root/app/assets/scrcpy-server"
(cd "$root/app/assets" && shasum -a 256 scrcpy-server > scrcpy-server.sha256)
echo "rebuilt scrcpy-server from $tag"
cat "$root/app/assets/scrcpy-server.sha256"
