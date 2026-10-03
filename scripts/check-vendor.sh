#!/usr/bin/env bash
# Check the vendored X_eTaL: the CLI builds, answers, reports the
# vendored commit, runs a demo and imports a standard library.
#   scripts/check-vendor.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
xetal="$("$root/scripts/build-xetal.sh")"
sha="$(sed -n 's/^commit = "\(.......\).*/\1/p' "$root/vendor/xetal/VENDORED")"
got="$("$xetal" eval -e "'+ r_/_2 2 3 r_eshape r_ange 6")"
[ "$got" = "6 15" ] || { echo "check-vendor: eval gave '$got', expected '6 15'" >&2; exit 1; }
version="$("$xetal" --version)"
case "$version" in *"$sha"*) ;; *) echo "check-vendor: xetal --version does not name $sha" >&2; echo "$version" >&2; exit 1 ;; esac
"$xetal" run "$root/vendor/xetal/demos/life.xtl" >/dev/null
got="$("$xetal" eval -e '"s:" u_se< "Stats"
s:m_ean 1 2 3 4')"
[ "$got" = "2.5" ] || { echo "check-vendor: Stats gave '$got', expected '2.5'" >&2; exit 1; }
echo "check-vendor: ok ($sha)"
