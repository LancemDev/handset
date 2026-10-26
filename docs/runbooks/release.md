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
3. The workflow runs again on the merge, finds no changesets left, tags the new version `vX.Y.Z` and
   creates a GitHub release with the changelog section as its notes.
4. In the same run a macOS job builds the release files with `scripts/package-release.sh` and attaches
   them to that release: `Handset-X.Y.Z-macos.zip` (universal app bundle),
   `handset-X.Y.Z-macos.tar.gz` (the command line tool, with the licence and notice) and `SHA256SUMS`.
   While the version is `0.x` it also marks the release as a pre-release.
5. Check the release page. With the signing secrets set (below) the app is Developer ID signed,
   notarised and stapled, so it opens without a Gatekeeper warning. Without them the files are
   ad-hoc signed and Gatekeeper blocks a downloaded copy until it is opened with right-click, Open
   (or `xattr -dr com.apple.quarantine`).

The first run after setup tags `v0.1.0`, the version already in `package.json`, without a version pull
request.

## One-time repository settings

- Settings > Actions > General > Workflow permissions: **Read and write**, and tick **Allow GitHub
  Actions to create and approve pull requests**. Without it the workflow cannot open the version PR.
- Tags created with the default `GITHUB_TOKEN` do not start other workflows, which is why the binary
  build is a job inside `release.yml` (using `needs` and the `published` output).

## Signing and notarising

`scripts/package-release.sh` signs with the hardened runtime when `CODESIGN_IDENTITY` is set, and
notarises (and staples the app) when notary credentials are set too. Both the app and the command line
tool are signed; one notarisation submission covers both. Without any of it the build is ad hoc.

You need a paid Apple Developer account, a **Developer ID Application** certificate, and notarisation
credentials. Keep all of it out of the repository.

Locally:

```sh
# one-off: store notarisation credentials in the keychain (prompts for an app-specific password)
xcrun notarytool store-credentials handset-notary --apple-id you@example.com --team-id TEAMID

CODESIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" NOTARY_PROFILE=handset-notary \
  scripts/package-release.sh
```

If the keychain holds several certificates with the same name, pass the SHA-1 hash from
`security find-identity -v -p codesigning` instead of the name.

In CI, add these repository secrets (Settings > Secrets and variables > Actions). The build job reads
them and falls back to an ad-hoc build when they are missing:

| Secret | Content |
|---|---|
| `MACOS_CERT_P12` | The Developer ID Application certificate with its private key, exported from Keychain Access as `.p12`, base64 encoded (`base64 -i cert.p12 \| pbcopy`) |
| `MACOS_CERT_PASSWORD` | The password chosen when exporting the `.p12` |
| `MACOS_CODESIGN_IDENTITY` | The certificate name, `Developer ID Application: Your Name (TEAMID)` |
| `NOTARY_KEY_P8` | An App Store Connect API key (`AuthKey_XXXX.p8`, base64 encoded) |
| `NOTARY_KEY_ID` | That key's ID |
| `NOTARY_ISSUER` | The issuer ID shown on the App Store Connect keys page |

Check a finished release with `spctl -a -vv Handset.app` (expect `accepted`, `source=Notarized Developer ID`)
and `xcrun stapler validate Handset.app`.

## Trying the build without releasing

Run the `release` workflow by hand (Actions > release > Run workflow, or `gh workflow run release.yml`).
It builds the same files and keeps them as the `handset-release-files` artifact, and publishes nothing.
Locally, `scripts/package-release.sh` writes them to `dist/release/` (needs both Rust targets,
`rustup target add aarch64-apple-darwin x86_64-apple-darwin`).

## If something goes wrong

- *The version pull request is stale:* push anything to `main`; the workflow rebuilds it.
- *Wrong bump:* edit the changeset file in the version pull request's base (or add a new changeset with
  the bigger bump) and let the pull request regenerate.
- *Versions disagree:* run `npm run version` locally with no changesets pending: it only re-syncs
  `app/Cargo.toml` from `package.json`.
