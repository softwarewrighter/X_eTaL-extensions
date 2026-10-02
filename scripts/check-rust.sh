#!/usr/bin/env bash
# This repo's Rust: formatted, clippy-clean (warnings are errors), and
# every test passing.
#   scripts/check-rust.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
cargo fmt --all --check
# vendor/ is never edited (xetal-x compiles some of its files as modules)
git diff --quiet -- vendor || { echo "check-rust: vendor/ has changes; never edit it" >&2; git status --short vendor >&2; exit 1; }
cargo clippy -q --workspace --all-targets -- -D warnings
cargo test -q --workspace 2>&1 | grep -E '^(test result|error|failures)' | grep -v ' 0 passed; 0 failed' || true
cargo test -q --workspace >/dev/null 2>&1 || { cargo test --workspace; exit 1; }
echo "check-rust: ok"
