# 0003. Reuse the unmodified scrcpy server

- Status: accepted
- Date: 2026-10-25

## Context

The phone side of screen mirroring (capturing and encoding the display, injecting input, capturing
audio) is hard to write and already solved by the scrcpy server, which is open source under Apache-2.0
and documents its protocol.

## Decision

We will embed the scrcpy server in the binary (`include_bytes!`), push it to the phone on connect, and
talk to it over adb port forwards. The server is not modified. Handset pins one exact version
(`SERVER_VERSION`, currently 5.0.1), because the server refuses clients with a different version string.

The embedded file is built from the `v5.0.1` tag (commit `a60891a`) with `./gradlew assembleDebug`
(see `docs/runbooks/rebuild-scrcpy-server.md`). It is a debug build, 998,542 bytes, with
SHA-256 `45863d316ff8a29c83e057052fada20bacf4646a1d31711831a87eadd222fb04`, recorded in `app/assets/scrcpy-server.sha256`.

## Alternatives considered

- **Write our own phone-side agent.** A large amount of Android-specific work for no benefit.
- **Download the server at run time.** Needs network access and a verification step on every machine.
- **Use the official release artifact instead of a local build.** Preferable, see below.

## Consequences

- Updating scrcpy means rebuilding or fetching the new server, bumping `SERVER_VERSION`, and
  reviewing the protocol for changes.
- The binary carries a roughly 1 MB asset, and we must keep the NOTICE and licence text in the repository.
- Follow-up: replace the local debug build with the official release artifact, fetched at build time and
  checked against a pinned checksum.
