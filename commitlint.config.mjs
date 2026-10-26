// Conventional Commits, with the scopes from .commit-scopes.
// The local hook in .githooks/commit-msg enforces the same rules without needing Node.
import fs from 'node:fs';

const scopes = fs
  .readFileSync(new URL('./.commit-scopes', import.meta.url), 'utf8')
  .split('\n')
  .map((line) => line.trim())
  .filter((line) => line && !line.startsWith('#'));

export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'scope-enum': [2, 'always', scopes],
    'header-max-length': [2, 'always', 72],
  },
};
