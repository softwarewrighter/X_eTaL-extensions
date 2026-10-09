#!/usr/bin/env bash
# The committed site, pages/, is what a fresh build makes: built into
# work/pages-check and compared, ignoring only the commit it was built
# at. Fails with the difference and the fix (just pages).
#   scripts/check-pages.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# built in a temporary directory outside the repository (never in work/,
# which is never erased: CLAUDE.md rule 18)
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fresh="$tmp/pages"
PAGES_DIR="$fresh" "$root/scripts/build-pages.sh" >/dev/null
# Lines that differ only in an inferred type are ignored: xetal doc types
# some functions differently from run to run (docs/xetal-asks.md, E9).
# A new or renamed function still changes other lines (its source page,
# its anchors), so the check still catches a stale site.
types='class="type"|<td>function</td>|","function","'
if ! diff -r -I 'built at [0-9a-f]*' -I "$types" "$root/pages" "$fresh" >"$tmp/pages-check.diff"; then
  head -40 "$tmp/pages-check.diff"
  echo "check-pages: pages/ is out of date; run just pages and commit it" >&2
  exit 1
fi
echo "check-pages: pages/ is up to date"
