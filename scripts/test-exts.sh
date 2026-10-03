#!/usr/bin/env bash
# Every extension's reg-rs tests (their Rust tests run in check-rust.sh),
# and the layout every extension must have (plan A7).
#   scripts/test-exts.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
n=0
for dir in "$root"/extensions/*/; do
  name="$(basename "$dir")"
  for need in extension.toml justfile rust/Cargo.toml lib tests docs/README.md demos; do
    [ -e "$dir/$need" ] || { echo "test-exts: extensions/$name has no $need" >&2; exit 1; }
  done
  out="$("$root/scripts/reg-ext.sh" "$name" run 2>&1)" || { echo "$out"; "$root/scripts/reg-ext.sh" "$name" run -vv; exit 1; }
  echo "test-exts: $name: $(echo "$out" | tail -1)"
  n=$((n + 1))
done
echo "test-exts: ok ($n extensions)"
