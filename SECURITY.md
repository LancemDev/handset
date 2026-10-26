# Security policy

## Supported versions

Handset has not had a stable release yet. Fixes are made on `main`.

## Reporting a vulnerability

Please do not open a public issue. Use GitHub's
[private vulnerability reporting](https://github.com/davidamunga/handset/security/advisories/new)
for this repository, and include steps to reproduce.

## Things worth knowing

- Handset controls a phone over adb. Anyone who can run adb against your phone can already do what
  Handset does, so protect your adb keys and only authorise computers you trust.
- The scrcpy server runs on the phone with shell privileges. It is embedded unmodified and pinned to a
  known version (see [ADR 0003](docs/adr/0003-reuse-the-unmodified-scrcpy-server.md)).
- Handset talks to the server through local adb port forwards on `127.0.0.1`.
