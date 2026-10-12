#!/usr/bin/env bash
# Publish the tower defense pieces demos/game-assets.xtl made (in
# extensions/blender/work/td/site) on the site, at pages/blender/td/:
# each piece's .glb and preview .png, index.html (the page that shows
# them) and manifest.toml, its commit line set to the commit the files
# were made at (HEAD, "+changes" when extensions/blender had changes
# not yet committed). Files are written over, never deleted. The pages
# build keeps this folder as it is (scripts/build-pages.sh): a demo
# makes it, not the build.
#   scripts/publish-td.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
src="$root/extensions/blender/work/td/site"
dest="$root/pages/blender/td"
[ -f "$src/manifest.toml" ] || { echo "publish-td: nothing made yet: just demo blender game-assets" >&2; exit 1; }
commit="$(git -C "$root" rev-parse HEAD)"
if [ -n "$(git -C "$root" status --porcelain -- extensions/blender)" ]; then commit="$commit+changes"; fi
mkdir -p "$dest"
cp "$src"/*.glb "$src"/*.png "$src/index.html" "$dest/"
sed "s/^commit = .*/commit = \"$commit\"/" "$src/manifest.toml" > "$dest/manifest.toml"
echo "publish-td: $(ls "$dest" | wc -l | tr -d ' ') files in pages/blender/td (commit $commit); commit pages/ and push to publish"
