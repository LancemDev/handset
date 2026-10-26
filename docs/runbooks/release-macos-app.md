# Release the macOS app

Superseded. Releases are cut by running the `release` workflow by hand: see [Release a version](release.md), which covers
changesets, the signed and notarised build, the GitHub release and the Homebrew cask.

To build a local copy by hand: `scripts/package-macos.sh` (ad hoc signed), or `scripts/package-release.sh`
for the full set of release files.
