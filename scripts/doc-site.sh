#!/usr/bin/env bash
# Build the cross-referenced documentation site (X_eTaL's `xetal doc
# --out`) into pages/doc: the binding macro (lib/Ffi.xtlm), every
# extension's facade, the libraries the demos share and every demo --
# each definition with its type, its `##` doc comment, its source drawn
# decorated with every name linked, the built-ins among them. Run by
# scripts/build-pages.sh (just pages); `just doc` builds it alone.
#   scripts/doc-site.sh          # into pages/doc
#   DOC_OUT=DIR scripts/doc-site.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
xetal="$("$root/scripts/build-xetal.sh")"
out="${DOC_OUT:-pages/doc}"
rm -rf "$out"
# facades and the shared demo libraries are found by name
path="lib"
for d in extensions/*/lib extensions/*/demos; do path="$path:$d"; done
files=(lib/*.xtlm extensions/*/lib/*.xtl extensions/*/demos/*.xtl)
XETAL_PATH="$path" "$xetal" doc --out "$out" "${files[@]}" > /dev/null
echo "doc: $(find "$out" -name '*.html' | wc -l | tr -d ' ') pages in $out"
