#!/usr/bin/env bash
# The machine is free to build and test: nothing else is compiling in
# this repository's target/ or rendering with Blender. A Blender render
# (just demo blender game-assets, just videos) takes every core for
# minutes, and a gate beside it ran several times slower; a trunk serve
# rebuilds on every change and holds the cargo lock; so does another
# session's cargo. Run first by the gate, check-quick and just pages.
# Adapted from ../X_eTaL/scripts/check-busy.sh.
#   scripts/check-busy.sh          # fail when something else is busy
#   scripts/check-busy.sh --stop   # stop a trunk serve of this repository
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
serving="$(pgrep -f "trunk serve" || true)"
if [ "${1:-}" = "--stop" ]; then
  if [ -n "$serving" ]; then kill $serving && echo "check-busy: stopped trunk serve ($serving)"; else echo "check-busy: no trunk serve running"; fi
  exit 0
fi
if [ -n "$serving" ]; then
  echo "check-busy: a trunk serve is running (pid $serving); it rebuilds on every change and holds the cargo lock: just stop-serve" >&2
  exit 1
fi
rendering="$(pgrep -f "Blender.app/Contents/MacOS/Blender|blender --background" | head -1 || true)"
if [ -n "$rendering" ]; then
  echo "check-busy: Blender is running (pid $rendering, $(ps -o etime= -p "$rendering" | tr -d ' ') old), a render that takes every core: wait for it" >&2
  exit 1
fi
# another cargo holding one of this target's locks (run before the
# gate starts any cargo of its own)
for lock in "$root/target/debug/.cargo-lock" "$root/target/release/.cargo-lock"; do
  holder="$(lsof -t "$lock" 2>/dev/null | head -1 || true)"
  if [ -n "$holder" ]; then
    echo "check-busy: another cargo (pid $holder, $(ps -o etime= -p "$holder" | tr -d ' ') old) holds ${lock#$root/}: wait for it or stop it" >&2
    exit 1
  fi
done
echo "machine: free to build"
