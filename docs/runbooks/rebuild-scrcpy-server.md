# Rebuild the scrcpy server

The phone-side server is embedded as `app/assets/scrcpy-server` (see
[ADR 0003](../adr/0003-reuse-the-unmodified-scrcpy-server.md)). Rebuild it when you bump the pinned
version, or to check that the file in the repository matches the source.

## Prerequisites

- JDK 17 (`java -version`)
- Android SDK with the platform and build-tools that scrcpy's `build.gradle` asks for (`ANDROID_HOME`
  set; Gradle offers to install missing pieces if the licences are accepted)
- git and network access

## Steps

1. Pick the tag. The current pin is `v5.0.1`.
2. Run the script:

   ```sh
   scripts/build-scrcpy-server.sh v5.0.1
   ```

   It clones the tag, runs `./gradlew assembleDebug`, copies the APK to `app/assets/scrcpy-server` and
   rewrites `app/assets/scrcpy-server.sha256`.
3. If you changed the version:
   - set `SERVER_VERSION` in `app/src/server.rs` to the same string (the server rejects any other),
   - check the scrcpy release notes and `doc/develop.md` for protocol changes (video header, control
     message layout, audio options) and adjust `server.rs` and `control.rs` if needed,
   - update ADR 0003 (or add a new ADR) with the tag, commit and checksum.
4. Run the app against a phone and confirm video, touch and audio still work.
5. Commit the asset, checksum, version bump and ADR change together.

## Verifying without rebuilding

```sh
cd app/assets && shasum -a 256 -c scrcpy-server.sha256
```

## Known limits

- The result is a debug build and about 1 MB. Moving to the official release artifact, fetched at build
  time and checked against a pinned checksum, is a recorded follow-up.
- The script has been written from the manual steps used for the current asset; re-run it before relying
  on it for a version bump.
