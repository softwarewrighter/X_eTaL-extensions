#!/usr/bin/env bash
# Build the cross-referenced documentation site (X_eTaL's `xetal doc
# --out`) into pages/doc: the binding macro (lib/Ffi.xtlm), every
# extension's facade, the libraries the demos share and every demo --
# each definition with its type, its `##` doc comment, its source drawn
# decorated with every name linked, the built-ins among them. Run by
# scripts/build-pages.sh (just pages); `just doc` builds it alone. Built
# in a temporary directory, then copied only where something really
# changed (scripts/sync-pages.py); nothing is deleted.
#   scripts/doc-site.sh          # into pages/doc
#   DOC_OUT=DIR scripts/doc-site.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
xetal="$("$root/scripts/build-xetal.sh")"
dest="${DOC_OUT:-pages/doc}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
out="$tmp/doc"
# facades and the shared demo libraries are found by name
path="lib"
for d in extensions/*/lib extensions/*/demos; do path="$path:$d"; done
# and the shared libraries used here (X_eTaL-libraries at LIBRARIES_COMMIT)
path="$path:$("$root/scripts/libraries.sh")"
# the binding macro, then each extension's facade followed by its demos
# (the sidebar groups files by directory, in this order)
files=(lib/*.xtlm)
for d in extensions/*; do
  for f in "$d"/lib/*.xtl "$d"/demos/*.xtl; do [ -e "$f" ] && files+=("$f"); done
done
XETAL_PATH="$path" "$xetal" doc --out "$out" "${files[@]}" > /dev/null
mkdir -p "$dest"
"$root/scripts/sync-pages.py" "$out" "$dest" >&2
echo "doc: $(find "$out" -name '*.html' | wc -l | tr -d ' ') pages in $dest"
