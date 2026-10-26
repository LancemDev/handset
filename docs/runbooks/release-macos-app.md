# Release the macOS app

Builds `Handset.app`, checks it, and publishes it. Today the app is ad-hoc signed (see
[ADR 0010](../adr/0010-package-as-an-ad-hoc-signed-app-bundle.md)), so a release is for people who are
happy to approve it once in Gatekeeper.

## Before you start

- You are on `main`, up to date, with a clean working tree.
- A phone is connected, for the smoke test.
- `ffmpeg` is installed if you want to check recording.

## Steps

1. Move the `[Unreleased]` entries in `CHANGELOG.md` under a new version heading with today's date.
2. Set the same version in `app/Cargo.toml`. Commit both as `chore(repo): prepare vX.Y.Z`.
3. Run the checks:

   ```sh
   cd app
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```

4. Build the bundle:

   ```sh
   scripts/package-macos.sh
   codesign --verify --deep --strict dist/Handset.app
   ```

5. Smoke test with the phone connected:

   ```sh
   open dist/Handset.app
   ```

   Check that the window shows the phone, touch works, a screenshot saves and the toast appears, and
   (with `ffmpeg`) a recording has video and audio. Quit from the toolbar.
6. Zip it and tag the release:

   ```sh
   ditto -c -k --keepParent dist/Handset.app dist/Handset-X.Y.Z-macos.zip
   git tag -a vX.Y.Z -m "Handset X.Y.Z"
   git push origin vX.Y.Z
   ```

7. Create a GitHub release from the tag, paste the changelog section, and attach the zip. Say in the
   notes that the app is ad-hoc signed and how to open it (right-click, Open).

## Afterwards

Open a pull request that starts a fresh `[Unreleased]` section if the release commit did not.

## Known gaps

- No notarisation and no universal build yet; both are recorded in ADR 0010.
