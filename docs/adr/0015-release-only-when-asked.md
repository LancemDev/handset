# 0015. Release only when asked, and never as a pre-release

- Status: accepted
- Date: 2026-10-26

## Context

ADR 0012 had the release workflow tag and publish as soon as a "Version packages" pull request was merged,
and marked 0.x releases as pre-releases. Merging a pull request is a review step, not a decision to ship:
a release also notarises a build, publishes files and updates the Homebrew cask, and people install from it.

## Decision

Split the workflow in two. `version` opens and updates the version pull request on every push to `main`,
and merging it only bumps versions and the changelog. `release` is started by hand
(`workflow_dispatch`, with an optional dry run), checks that the version in `package.json` is not tagged yet
and has a changelog section, builds the signed and notarised files, tags `vX.Y.Z`, creates a normal GitHub
release (not a pre-release, marked latest) with the changelog section as notes, and bumps the Homebrew cask.

## Consequences

- Shipping is one deliberate action, and version pull requests can be merged and batched freely.
- `changeset tag` and the `published` output are no longer used; the tag comes from `gh release create`.
- A release needs the workflow run by a person with write access, from `main`.
- 0.x versions are announced like any other release, so the version number alone carries the early-stage
  message.

## Alternatives considered

- **Keep releasing on merge, and gate it with a protected environment that needs approval:** also manual, but
  the version PR still looks like it ships something and approval lives in a different place.
- **Tag by hand and release on the tag push:** tags made by hand are fine, but the signing secrets and checks
  would then run on any pushed tag.
