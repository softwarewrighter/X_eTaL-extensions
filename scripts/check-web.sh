#!/usr/bin/env bash
# The live pages' Rust (web/shell and every extensions/*/web): each is a
# workspace of its own; formatted, clippy-clean, its native tests
# passing, and building for the browser (wasm32).
#   scripts/check-web.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
n=0
for dir in "$root/web/shell" "$root"/extensions/*/web; do
  [ -f "$dir/Cargo.toml" ] || continue
  (
    cd "$dir"
    cargo fmt --check
    cargo clippy -q --all-targets -- -D warnings
    cargo test -q >/dev/null 2>&1 || cargo test
    cargo check -q --target wasm32-unknown-unknown
  ) || { echo "check-web: ${dir#$root/} failed" >&2; exit 1; }
  n=$((n + 1))
done
echo "check-web: ok ($n crates)"
