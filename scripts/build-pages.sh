#!/usr/bin/env bash
# Build the site into pages/, which is committed: the Pages workflow
# publishes that folder as it is. The site shows each extension's demos
# running (the recordings `just videos` makes) with the commands to run
# them; extensions are native, so the site does not run them.
#   scripts/build-pages.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
rm -rf "$root/pages"
mkdir -p "$root/pages"
touch "$root/pages/.nojekyll"
cp "$root/images/modern-xetal-logo.jpg" "$root/images/favicon.ico" "$root/web/shell/shell.css" "$root/pages/"
"$root/scripts/build-catalog.py"
echo "pages/ built; commit it (git add -A pages/) and push to publish."
