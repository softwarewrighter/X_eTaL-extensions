#!/usr/bin/env bash
# Publish the three extension's live page on the site: built with trunk
# (X_eTaL and the extension as WebAssembly, the player and Three.js
# beside it) and copied to pages/three/live, where scripts/build-pages.sh
# keeps it (as it keeps pages/blender/td). Commit pages/ and push.
#   scripts/publish-three.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
"$root/scripts/xetal.sh" >/dev/null
command -v trunk >/dev/null || { echo "publish-three: trunk not found" >&2; exit 127; }
web="$root/extensions/three/web"
dest="$root/pages/three/live"
# file names without hashes, so a new build writes over the last
(cd "$web" && trunk build --release --filehash false --public-url ./ > /dev/null)
mkdir -p "$dest"
cp -R "$web/dist/." "$dest/"
echo "publish-three: $(find "$dest" -type f | wc -l | tr -d ' ') files in pages/three/live; commit pages/ and push to publish"
