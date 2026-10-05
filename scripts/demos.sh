#!/usr/bin/env bash
# The demos, and running one.
#   scripts/demos.sh                 # list them: EXT NAME -- what it is
#   scripts/demos.sh EXT NAME [ARGS] # run extensions/EXT/demos/NAME.xtl
# A demo runs with xetal-x and every extension loaded, from its
# extension's directory (its data paths are relative to it); pictures
# go to its work/draw/. Window demos open a window.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# X_eTaL's clone (work/xetal) must exist: this repo's crates build on it
"$root/scripts/xetal.sh" >/dev/null
if [ "$#" -eq 0 ]; then
  for f in "$root"/extensions/*/demos/*.xtl; do
    # a capitalized file there is a library the demos share, not a demo
    case "$(basename "$f")" in [ABCDEFGHIJKLMNOPQRSTUVWXYZ]*) continue ;; esac
    ext="$(basename "$(dirname "$(dirname "$f")")")"
    name="$(basename "$f" .xtl)"
    what="$(sed -n '1s/^# *//p' "$f")"
    printf '%-8s %-12s %s\n' "$ext" "$name" "$what"
  done
  echo
  echo "run one: just demo EXT NAME   (record them: just videos)"
  exit 0
fi
ext="$1"; name="${2:?usage: just demo EXT NAME}"; shift 2
file="$root/extensions/$ext/demos/$name.xtl"
[ -f "$file" ] || { echo "no demo $ext $name (just demos lists them)" >&2; exit 1; }
# release: demos like the visualizer need its speed (a debug build
# draws about 9 frames a second)
(cd "$root" && cargo build -q --release --workspace)
cd "$root/extensions/$ext"
exec "$root/target/release/xetal-x" --ext "$root/extensions" run --draw work/draw "$@" "demos/$name.xtl"
