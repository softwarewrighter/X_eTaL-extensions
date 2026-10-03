#!/usr/bin/env bash
# The pre-commit gate: the vendored X_eTaL (scripts/check-vendor.sh),
# this repo's Rust (scripts/check-rust.sh: fmt, clippy, tests), the
# bridge host matching the vendored CLI (scripts/check-xetal-x.sh),
# every extension's reg-rs tests (scripts/test-exts.sh), then
# ASCII-only markdown for the docs we own.
#   scripts/gate.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
"$root/scripts/check-vendor.sh"
"$root/scripts/check-rust.sh"
"$root/scripts/check-xetal-x.sh"
"$root/scripts/test-exts.sh"
md=(README.md CHANGES.md docs/plan.md docs/xetal-asks.md)
for f in docs/*.md extensions/*/docs/*.md; do [ -e "$f" ] && md+=("$f"); done
for f in "${md[@]}"; do sw-markdown-checker -f "$f" >/dev/null || { sw-markdown-checker -f "$f"; exit 1; }; done
echo "gate: ok"
