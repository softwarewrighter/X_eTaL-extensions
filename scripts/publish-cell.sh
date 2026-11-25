#!/usr/bin/env bash
# The assembly cell's live page: built with trunk (the simulator and
# X_eTaL as WebAssembly, the three extension's cell player and Three.js
# beside them), the cell's models added (just ext cell models makes
# them), into extensions/cell/web/dist; with --publish also copied to
# pages/cell/live, where scripts/build-pages.sh keeps it. Commit pages/
# and push.
#   scripts/publish-cell.sh            # build extensions/cell/web/dist
#   scripts/publish-cell.sh --publish  # and copy it to pages/cell/live
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
"$root/scripts/xetal.sh" >/dev/null
command -v trunk >/dev/null || { echo "publish-cell: trunk not found (cargo install --locked trunk)" >&2; exit 127; }
web="$root/extensions/cell/web"
models="$root/extensions/cell/work/cell/models"
[ -f "$models/arm.glb" ] || { echo "publish-cell: no models yet: just ext cell models" >&2; exit 1; }
# the build stamp (host, commit, time) is made when the shell is built:
# build it again, so the About dialog says this build
touch "$root/web/shell/build.rs"
# file names without hashes, so a new build writes over the last
(cd "$web" && trunk build --release --filehash false --public-url ./ > /dev/null)
mkdir -p "$web/dist/models"
cp "$models"/*.glb "$web/dist/models/"
echo "publish-cell: $(find "$web/dist" -type f | wc -l | tr -d ' ') files in extensions/cell/web/dist"
[ "${1:-}" = --publish ] || exit 0
dest="$root/pages/cell/live"
mkdir -p "$dest"
cp -R "$web/dist/." "$dest/"
# trunk's lock file is for its own builds, not the site
rm -f "$dest/.trunk-lock"
echo "publish-cell: copied to pages/cell/live; commit pages/ and push to publish"
