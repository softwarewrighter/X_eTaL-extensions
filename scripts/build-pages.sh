#!/usr/bin/env bash
# Build the site into pages/, which is committed: the Pages workflow
# publishes that folder as it is. The site shows each extension's demos
# running (the recordings `just videos` makes) with the commands to run
# them; extensions are native, so the site does not run them.
# It is built in a temporary directory, then copied into pages/ only
# where something really changed (scripts/sync-pages.py), so a rebuild
# adds nothing to the repository when nothing changed; nothing in pages/
# is deleted (a page no longer made is listed: ask first, rule 18).
#   scripts/build-pages.sh   # PAGES_DIR=DIR builds into an empty DIR instead (scripts/check-pages.sh)
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [ -n "${PAGES_DIR:-}" ]; then
  out="$PAGES_DIR"
else
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  out="$tmp/pages"
fi
mkdir -p "$out"
touch "$out/.nojekyll"
cp "$root/images/xetal-logo-red.png" "$root/images/favicon.ico" "$root/web/shell/shell.css" "$out/"
PAGES_DIR="$out" "$root/scripts/build-catalog.py"
# the cross-referenced docs of every facade, shared library and demo
DOC_OUT="$out/doc" "$root/scripts/doc-site.sh" > /dev/null
[ -n "${PAGES_DIR:-}" ] && exit 0
"$root/scripts/sync-pages.py" "$out" "$root/pages"
echo "pages/ built; commit it (git add pages/) and push to publish."
