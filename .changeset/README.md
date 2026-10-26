# Changesets

Every pull request that changes what a user sees adds one file here, a *changeset*. Releases collect
them into the version number and `CHANGELOG.md`.

```sh
npm install        # once
npm run changeset  # answer the prompts, commit the file it writes
```

Pick the bump for the change: `patch` for a fix, `minor` for a new feature, `major` for something that
breaks how people use it (while Handset is `0.x`, a breaking change is a `minor`). Write the summary for a
user of the app, in the past tense or as a short statement, not as a commit message.

No changeset is needed for docs, tests, CI or refactors that do not change behaviour.

How a release happens is in [`docs/runbooks/release.md`](../docs/runbooks/release.md).
