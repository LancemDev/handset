# Release a version

Releases use [Changesets](https://github.com/changesets/changesets). Handset is a Rust crate, so the
root `package.json` exists only to carry the version; `scripts/sync-version.mjs` copies it into
`app/Cargo.toml` and `app/Cargo.lock`, which is where the app and its bundle read it from.

## Day to day: add a changeset

Any pull request that changes what a user sees adds a changeset:

```sh
npm install        # once
npm run changeset  # pick patch, minor or major and write the summary
```

Commit the generated `.changeset/*.md` with the pull request. Docs, tests, CI and behaviour-neutral
refactors do not need one. While Handset is `0.x`, use `minor` for anything that breaks existing use.

## Cutting a release

1. Pushes to `main` run the `release` workflow. While changesets are pending it opens or updates a pull
   request titled `chore(release): version packages`. It bumps `package.json`, `app/Cargo.toml` and
   `app/Cargo.lock` and writes the new section of `CHANGELOG.md`.
2. Read that pull request. Edit `CHANGELOG.md` there if a summary needs rewording, then merge it.
3. The workflow runs again on the merge, finds no changesets left, and tags the new version `vX.Y.Z`
   and creates a GitHub release with the changelog section as its notes.
4. Check the release page, then build the binaries from that tag (see the packaging runbook once it
   exists; today use `scripts/package-macos.sh`).

The first run after setup tags `v0.1.0`, the version already in `package.json`, without a version pull
request.

## One-time repository settings

- Settings > Actions > General > Workflow permissions: **Read and write**, and tick **Allow GitHub
  Actions to create and approve pull requests**. Without it the workflow cannot open the version PR.
- Tags created with the default `GITHUB_TOKEN` do not start other workflows. A build that must run on
  each release has to be a job in `release.yml` (using `needs` and the `published` output), or the
  workflow needs a personal access token or GitHub App token.

## If something goes wrong

- *The version pull request is stale:* push anything to `main`; the workflow rebuilds it.
- *Wrong bump:* edit the changeset file in the version pull request's base (or add a new changeset with
  the bigger bump) and let the pull request regenerate.
- *Versions disagree:* run `npm run version` locally with no changesets pending: it only re-syncs
  `app/Cargo.toml` from `package.json`.
