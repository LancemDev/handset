# Contributing to Handset

Thanks for taking a look. Handset is a young project, so small, focused changes are the easiest to
review and merge.

## Getting set up

You need:

- a recent stable Rust toolchain (`rustup update stable`)
- `adb` from the Android platform-tools
- `ffmpeg` (optional, only for recording)
- an Android phone with USB debugging enabled (most changes need a real device to test)

```sh
git clone https://github.com/davidamunga/handset.git
cd handset
scripts/install-hooks.sh        # enables the commit message check
cd app
cargo run --release
```

## Before you open a pull request

```sh
cd app
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Run the app against a real phone if you touched anything in the video, input, audio or capture paths,
and say so in the pull request.

If the change is something a user would notice (a feature, a fix, a changed option), add a changeset:
`npm install` once, then `npm run changeset`, and commit the file it writes. Docs, tests, CI and
refactors do not need one. See [the release runbook](docs/runbooks/release.md).

## Commit messages and pull request titles

We use [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): subject`.

- Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
- Scopes are listed in [`.commit-scopes`](.commit-scopes). The scope is optional.
- Keep the subject in lower case, in the imperative mood, with no trailing full stop.

`scripts/install-hooks.sh` installs a local `commit-msg` hook that checks this. CI checks commits and
pull request titles again.

## Decisions and runbooks

- Architectural decisions are recorded as short ADRs in [`docs/adr`](docs/adr). If your change
  introduces a new dependency, changes how Handset talks to the phone, or reverses an earlier decision,
  add or update an ADR in the same pull request.
- Repeatable procedures live in [`docs/runbooks`](docs/runbooks).

## Reporting bugs

Use the bug report template and include your macOS version, phone model and Android version, and the
output of `HANDSET_DEBUG=1 handset`.

## Code of conduct

Be kind. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
