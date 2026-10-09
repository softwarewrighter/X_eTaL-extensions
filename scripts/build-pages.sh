#!/usr/bin/env bash
# Build the site into pages/, which is committed: the Pages workflow
# publishes that folder as it is. The site shows each extension's demos
# running (the recordings `just videos` makes) with the commands to run
# them; extensions are native, so the site does not run them.
#   scripts/build-pages.sh   # PAGES_DIR=DIR builds elsewhere (scripts/check-pages.sh)
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${PAGES_DIR:-$root/pages}"
rm -rf "$out"
mkdir -p "$out"
touch "$out/.nojekyll"
cp "$root/images/xetal-logo-red.png" "$root/images/favicon.ico" "$root/web/shell/shell.css" "$out/"
PAGES_DIR="$out" "$root/scripts/build-catalog.py"
# the cross-referenced docs of every facade, shared library and demo
DOC_OUT="$out/doc" "$root/scripts/doc-site.sh" > /dev/null
[ -n "${PAGES_DIR:-}" ] && exit 0
echo "pages/ built; commit it (git add -A pages/) and push to publish."
