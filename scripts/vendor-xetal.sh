#!/usr/bin/env bash
# Vendor X_eTaL: replace vendor/xetal/ with a snapshot of a COMMITTED
# ref of the X_eTaL repository (never its working tree), and record
# which one in vendor/xetal/VENDORED.
#   scripts/vendor-xetal.sh            # HEAD of ../X_eTaL
#   scripts/vendor-xetal.sh REF        # any ref git rev-parse accepts
#   XETAL_REPO=/path scripts/vendor-xetal.sh
# Taken: components/ (the crates), lib/ (the standard libraries, which
# xetal-libs embeds), demos/ and userlibs/ (which xetal-web embeds),
# .cargo/, LICENSE and COPYRIGHT. Not taken: tests' data (spec/, reg/),
# docs, images, pages.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repo="${XETAL_REPO:-$root/../X_eTaL}"
ref="${1:-HEAD}"
paths=(components lib userlibs demos .cargo LICENSE COPYRIGHT)
sha="$(git -C "$repo" rev-parse --verify "$ref^{commit}")"
dest="$root/vendor/xetal"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
git -C "$repo" archive --format=tar "$sha" -- "${paths[@]}" | tar -x -C "$tmp"
# Each component's Cargo.lock is kept: builds use the versions X_eTaL tested.
mkdir -p "$dest"
rsync -a --delete --exclude=VENDORED "$tmp/" "$dest/"
{
  echo "repository = \"https://github.com/softwarewrighter/X_eTaL\""
  echo "commit = \"$sha\""
  echo "subject = \"$(git -C "$repo" log -1 --format=%s "$sha" | sed 's/"/\\"/g')\""
  echo "committed = \"$(git -C "$repo" log -1 --format=%cI "$sha")\""
  echo "vendored = \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\""
} > "$dest/VENDORED"
echo "vendored X_eTaL ${sha:0:7} into vendor/xetal/"
cat "$dest/VENDORED"
