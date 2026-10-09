#!/usr/bin/env bash
# This repo's Rust: formatted, clippy-clean (warnings are errors), and
# every test passing.
#   scripts/check-rust.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# X_eTaL's clone (work/xetal) must exist: this repo's crates build on it
"$root/scripts/xetal.sh" >/dev/null
cd "$root"
cargo fmt --all --check
cargo clippy -q --workspace --all-targets -- -D warnings
# the tests once: their summaries, or everything when one fails
out="$(cargo test -q --workspace 2>&1)" || { echo "$out"; exit 1; }
echo "$out" | grep -E '^(test result|error|failures)' | grep -v ' 0 passed; 0 failed' || true
echo "check-rust: ok"
