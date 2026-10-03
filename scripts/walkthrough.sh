#!/usr/bin/env bash
# A fresh user's first run (research3's "release candidate as a
# stranger"): clone this repository into a temporary directory, build
# it with nothing but Rust (X_eTaL is vendored), and run release 1's
# programs -- hello's tour, the clock, the sqlite data notebook --
# checking each against its committed golden.
#   scripts/walkthrough.sh                 # clone this checkout's HEAD
#   scripts/walkthrough.sh URL             # clone URL (e.g. the GitHub repository)
#   KEEP=1 scripts/walkthrough.sh          # keep the clone to look around
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
src="${1:-$root}"
dir="$(mktemp -d)"
[ -n "${KEEP:-}" ] || trap 'rm -rf "$dir"' EXIT
step() { printf '\n==> %s\n' "$*"; }

step "clone $src"
git clone -q "$src" "$dir/X_eTaL-extensions"
cd "$dir/X_eTaL-extensions"
git log -1 --format='    at %h %s'

step "build (cargo build --workspace: the vendored X_eTaL, the ABI, the extensions, xetal-x)"
CARGO_TARGET_DIR="$dir/target" cargo build -q --workspace
xx="$dir/target/debug/xetal-x"

step "what is loaded"
"$xx" --ext extensions --ext-list | grep -E '^[a-z]'

check() { # NAME DIR EXPECTED-FILE COMMAND...
  local name="$1" d="$2" want="$3"; shift 3
  local got
  got="$(cd "$d" && "$@" 2>/dev/null)"
  if [ "$got" = "$(cat "$want")" ]; then echo "    $name: as expected"; else
    echo "    $name: differs from $want"; diff <(echo "$got") "$want" | head -20; exit 1; fi
}

step "hello: the tour"
check "hello tour" extensions/hello extensions/hello/tests/hello-demo-tour.out \
  "$xx" --ext . run demos/tour.xtl

step "clock: the time now"
(cd extensions/clock && "$xx" --ext . eval -e '"ck:" u_se< "Clock"
ck:i_so @')

step "sqlite: the data notebook (CO2 at Mauna Loa)"
(cd extensions/sqlite && rm -rf work && "$xx" --ext . run --draw work/draw demos/notebook.xtl 2>/dev/null | tail -9)
check "notebook" extensions/sqlite <(sed '/^[0-9]* [0-9]* work\/draw/d' extensions/sqlite/tests/sqlite-demo-notebook.out) \
  sh -c "rm -rf work && '$xx' --ext . run --draw work/draw demos/notebook.xtl"

step "walkthrough: ok (clone in $dir${KEEP:+, kept})"
