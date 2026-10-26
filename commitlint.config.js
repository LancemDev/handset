// Conventional Commits, with the scopes from .commit-scopes.
// The local hook in .githooks/commit-msg enforces the same rules without needing Node.
const fs = require('node:fs');
const path = require('node:path');

const scopes = fs
  .readFileSync(path.join(__dirname, '.commit-scopes'), 'utf8')
  .split('\n')
  .map((line) => line.trim())
  .filter((line) => line && !line.startsWith('#'));

module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'scope-enum': [2, 'always', scopes],
    'header-max-length': [2, 'always', 72],
  },
};
