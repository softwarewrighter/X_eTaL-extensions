#!/usr/bin/env bash
# The committed site, pages/, is what a fresh build makes: built into
# work/pages-check and compared, ignoring only the commit it was built
# at. Fails with the difference and the fix (just pages).
#   scripts/check-pages.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fresh="$root/work/pages-check"
PAGES_DIR="$fresh" "$root/scripts/build-pages.sh" >/dev/null
if ! diff -r -I 'built at [0-9a-f]*' "$root/pages" "$fresh" >"$root/work/pages-check.diff"; then
  cat "$root/work/pages-check.diff" | head -40
  echo "check-pages: pages/ is out of date; run just pages and commit it" >&2
  exit 1
fi
echo "check-pages: pages/ is up to date"
