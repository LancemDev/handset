# 0012. Release with Changesets

- Status: accepted (the release step is superseded by [0015](0015-release-only-when-asked.md))
- Date: 2026-10-26

## Context

Handset needs a repeatable way to choose a version, write the changelog and tag a release before it is
shipped as a binary and through Homebrew. Commits already follow Conventional Commits, and the project is
a single Rust crate in a repository that otherwise needs no Node.

## Decision

Use Changesets. Each user-visible pull request adds a small `.changeset/*.md` file stating the bump and a
summary written for users. A workflow keeps a "Version packages" pull request up to date; merging it bumps
the version and writes `CHANGELOG.md`, and the next run tags `vX.Y.Z` and creates the GitHub release.

A root `package.json` (private, not published) holds the version, and `scripts/sync-version.mjs` copies it
into `app/Cargo.toml` and `app/Cargo.lock`. Node is needed only to run the release tooling, in CI and by
contributors who add a changeset.

## Consequences

- The changelog is written by people, per change, instead of derived from commit subjects.
- Contributors need Node once to add a changeset. The Rust build and the commit hook stay Node-free.
- Tags made with the default `GITHUB_TOKEN` do not trigger other workflows, so binary builds must live in
  the same workflow or use another token (see the release runbook).

## Alternatives considered

- **release-plz:** Rust-native and derives releases from commits, so no per-pull-request files, but release
  notes cannot be written by hand.
- **git-cliff and manual tags:** least tooling, most manual steps.
