#!/usr/bin/env bash
# A quick gate for commits that change no code -- recordings, docs, goldens'
# scripts and expectations, X_eTaL demo programs: every extension's reg-rs
# tests, docs/status.md, the site up to date, every export documented,
# American spellings and ASCII-only markdown (the gate's checks that such
# a change can affect). It refuses when the working tree changes code
# (Rust, Cargo files, scripts, the justfile, the pinned commits): run
# `just gate` then.
#   scripts/check-quick.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
cd "$root"
"$root/scripts/check-busy.sh"
code="$( { git diff --name-only HEAD; git ls-files --others --exclude-standard; } \
  | grep -E '\.rs$|Cargo\.(toml|lock)$|^scripts/|^justfile$|/justfile$|^XETAL_COMMIT$|^LIBRARIES_COMMIT$|^crates/|^web/' \
  | grep -v '^web/shell/Cargo\.lock$' || true)"
if [ -n "$code" ]; then
  echo "check-quick: code changed -- run just gate:" >&2
  echo "$code" | sed 's/^/  /' >&2
  exit 1
fi
# the goldens of the extensions the changes touch (scripts/affected.py)
exts="$("$root/scripts/affected.py" | sed -n 's/^exts: *//p')"
case "$exts" in
  all) "$root/scripts/test-exts.sh" ;;
  "") echo "test-exts: no extension changed" ;;
  # shellcheck disable=SC2086
  *) "$root/scripts/test-exts.sh" $exts ;;
esac
"$root/scripts/status.py" --check
"$root/scripts/check-pages.sh"
"$root/scripts/check-docs.py"
"$root/scripts/literate.sh" --check
"$root/scripts/check-spelling.py" --self-test
"$root/scripts/check-spelling.py"
md=(README.md CHANGES.md docs/plan.md docs/xetal-asks.md)
for f in docs/*.md extensions/*/docs/*.md web/*/README.md; do [ -e "$f" ] && md+=("$f"); done
for f in "${md[@]}"; do sw-markdown-checker -f "$f" >/dev/null || { sw-markdown-checker -f "$f"; exit 1; }; done
echo "check-quick: ok (no code changed)"
