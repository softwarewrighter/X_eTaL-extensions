#!/usr/bin/env bash
# Write a farm job's scene: run an X_eTaL program (farm/NAME.xtl) that
# writes the script Blender will run. From the repository's root, with
# the blender extension's scene libraries (its demos: NeonCity) on
# XETAL_PATH. A job's prepare command (farm/jobs.toml).
#   scripts/farm-scene.sh farm/city-film.xtl
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
"$root/scripts/xetal.sh" >/dev/null
file="${1:?usage: farm-scene.sh farm/NAME.xtl}"
cd "$root"
[ -f "$file" ] || { echo "farm-scene: no $file" >&2; exit 1; }
cargo build -q --release -p xetal-x -p xetal-ext-blender
XETAL_PATH="$root/extensions/blender/demos:$("$root/scripts/libraries.sh")"
export XETAL_PATH
# only the blender extension: xetal-x fails on an extension directory
# whose library is not built, and only blender's is built here
exec "$root/target/release/xetal-x" --ext "$root/extensions/blender" run "$file"
