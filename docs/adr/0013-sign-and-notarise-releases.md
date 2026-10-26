# 0013. Sign with Developer ID and notarise releases

- Status: accepted
- Date: 2026-10-26

## Context

ADR 0010 packaged the app with an ad-hoc signature, which runs on the build machine but is blocked by
Gatekeeper on any other Mac after download. Releases are now published as binaries and through a Homebrew
tap, so people other than the author will run them.

## Decision

Release builds are signed with a Developer ID Application certificate using the hardened runtime and a
secure timestamp, then notarised with `notarytool` and stapled. The command line tool is signed and
notarised in the same submission. `scripts/package-macos.sh` and `scripts/package-release.sh` do this when
`CODESIGN_IDENTITY` and notary credentials are present and fall back to the ad-hoc signature otherwise, so
local builds, forks and the manual dry run keep working. The release workflow takes the certificate and an
App Store Connect API key from repository secrets.

No entitlements are needed. A signed build was run against a phone with video and audio under the
hardened runtime.

## Consequences

- Downloads open without a Gatekeeper warning, and a Homebrew cask no longer needs a quarantine
  workaround.
- Releases depend on an Apple Developer account and six repository secrets.
- A new capability (for example microphone input) would need an entitlement and a Info.plist usage string.

## Alternatives considered

- **Stay ad hoc and document the right-click workaround:** free, but every user hits the warning.
- **Notarise with an Apple ID and app-specific password:** works, but an API key is revocable and not tied
  to a personal login.
