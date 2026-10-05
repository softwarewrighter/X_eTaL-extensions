#!/usr/bin/env bash
# Check the known-good X_eTaL: the clone in work/xetal is clean and at
# XETAL_COMMIT, and its CLI builds, answers, names that commit, runs a
# demo and imports a standard library.
#   scripts/check-vendor.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
xetal="$("$root/scripts/build-xetal.sh")"
commit="$(tr -d '[:space:]' < "$root/XETAL_COMMIT")"
sha="${commit:0:7}"
clone="$root/work/xetal"
[ "$(git -C "$clone" rev-parse HEAD)" = "$commit" ] || { echo "check-vendor: work/xetal is not at XETAL_COMMIT" >&2; exit 1; }
[ -z "$(git -C "$clone" status --porcelain --untracked-files=no)" ] || { echo "check-vendor: work/xetal has local changes; never edit it" >&2; git -C "$clone" status --short >&2; exit 1; }
got="$("$xetal" eval -e "'+ r_/_2 2 3 r_eshape r_ange 6")"
[ "$got" = "6 15" ] || { echo "check-vendor: eval gave '$got', expected '6 15'" >&2; exit 1; }
version="$("$xetal" --version)"
case "$version" in *"$sha"*) ;; *) echo "check-vendor: xetal --version does not name $sha" >&2; echo "$version" >&2; exit 1 ;; esac
"$xetal" run "$clone/demos/life.xtl" >/dev/null
got="$("$xetal" eval -e '"s:" u_se< "Stats"
s:m_ean 1 2 3 4')"
[ "$got" = "2.5" ] || { echo "check-vendor: Stats gave '$got', expected '2.5'" >&2; exit 1; }
echo "check-vendor: ok ($sha)"
