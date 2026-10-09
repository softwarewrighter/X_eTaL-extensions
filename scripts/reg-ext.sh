#!/usr/bin/env bash
# Run reg-rs for one extension, with its baselines in its own tests/.
#   scripts/reg-ext.sh NAME run              # build, then run every baseline
#   scripts/reg-ext.sh NAME <reg-rs args>    # any other reg-rs command, e.g.
#   scripts/reg-ext.sh hello create -t hello-list -c 'xetal-x --ext . --ext-list'
# Commands run from extensions/NAME/ with target/release (xetal-x, the
# extension libraries) first on PATH, so a baseline says `xetal-x`: the
# interpreter is several times faster built for release, and the voxel
# goldens play hundreds of frames. An extension whose tests never share
# a work directory says so with a file tests/parallel, and its tests run
# side by side (reg-rs --parallel).
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# X_eTaL's clone (work/xetal) must exist: this repo's crates build on it
# (REG_EXT_BUILT=1: the caller has done this and the build, as
# scripts/test-exts.sh does before running extensions side by side)
[ -n "${REG_EXT_BUILT:-}" ] || "$root/scripts/xetal.sh" >/dev/null
name="${1:?usage: reg-ext.sh NAME reg-rs-args...}"
shift
dir="$root/extensions/$name"
[ -f "$dir/extension.toml" ] || { echo "reg-ext: no extension $name" >&2; exit 1; }
command -v reg-rs >/dev/null || { echo "reg-rs not found on PATH" >&2; exit 127; }
[ -n "${REG_EXT_BUILT:-}" ] || (cd "$root" && cargo build -q --release --workspace)
export REG_RS_DATA_DIR="$dir/tests"
export PATH="$root/target/release:$PATH"
# Pictures a golden draws go to work/draw (gitignored), never the repo.
export XETAL_DRAW="$root/work/draw"
mkdir -p "$XETAL_DRAW"
unset XETAL_EXT_PATH XETAL_PATH
# the shared libraries used here (X_eTaL-libraries at LIBRARIES_COMMIT;
# XETAL_LIBS when the caller has fetched them)
XETAL_PATH="${XETAL_LIBS:-$("$root/scripts/libraries.sh")}"
export XETAL_PATH
cd "$dir"
if [ "${1:-}" = "run" ] && [ "$#" -eq 1 ]; then
  if ! ls "$REG_RS_DATA_DIR"/*.rgt >/dev/null 2>&1; then
    echo "reg-ext: $name has no reg-rs tests"
    exit 0
  fi
  par=()
  [ -f "$REG_RS_DATA_DIR/parallel" ] && par=(--parallel)
  exec reg-rs run -p .rgt ${par[@]+"${par[@]}"}
fi
exec reg-rs "$@"
