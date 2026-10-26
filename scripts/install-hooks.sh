#!/usr/bin/env bash
# Point git at the repository's hooks (currently: the commit message check).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
git config core.hooksPath .githooks
chmod +x .githooks/*
echo "git hooks enabled (core.hooksPath = .githooks)"
