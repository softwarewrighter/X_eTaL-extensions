#!/usr/bin/env bash
# This repo's Rust: formatted, clippy-clean (warnings are errors), and
# every test passing. With package names (the gate's plan,
# scripts/affected.py), only those packages are linted and tested, and
# their doc-tests are left to the full gate; with none, nothing is
# (formatting is always checked: it takes no time).
#   scripts/check-rust.sh              # the whole workspace, doc-tests too
#   scripts/check-rust.sh PACKAGE ...  # those packages, no doc-tests
#   scripts/check-rust.sh --none       # formatting only
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
# X_eTaL's clone (work/xetal) must exist: this repo's crates build on it
"$root/scripts/xetal.sh" >/dev/null
cd "$root"
cargo fmt --all --check
if [ "${1:-}" = --none ]; then
  echo "check-rust: formatted (no package changed)"
  exit 0
fi
if [ $# = 0 ]; then
  which=(--workspace)
  kinds=()
else
  which=()
  for p in "$@"; do which+=(-p "$p"); done
  kinds=(--lib --tests --bins)
fi
cargo clippy -q "${which[@]}" --all-targets -- -D warnings
# the tests once: their summaries, or everything when one fails
out="$(cargo test -q "${which[@]}" ${kinds[@]+"${kinds[@]}"} 2>&1)" || { echo "$out"; exit 1; }
echo "$out" | grep -E '^(test result|error|failures)' | grep -v ' 0 passed; 0 failed' || true
echo "check-rust: ok (${*:-the workspace})"
