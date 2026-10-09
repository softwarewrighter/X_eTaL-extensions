#!/usr/bin/env bash
# Get X_eTaL-libraries: clone it into work/libraries (gitignored) and check
# out the known-good commit in LIBRARIES_COMMIT, as scripts/xetal.sh does
# for X_eTaL. Prints the directories to put on XETAL_PATH (the src/
# directories of the libraries used here, colon-separated): the demos
# that use a shared library (the Rubik's cube solver, Eigencube) reach it
# there. Never edit the
# clone; move to a newer commit by changing LIBRARIES_COMMIT.
#   scripts/libraries.sh
#   LIBRARIES_SOURCE=../X_eTaL-libraries scripts/libraries.sh   # clone from a local checkout
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
commit="$(tr -d '[:space:]' < "$root/LIBRARIES_COMMIT")"
source="${LIBRARIES_SOURCE:-https://github.com/softwarewrighter/X_eTaL-libraries.git}"
clone="$root/work/libraries"

if [ ! -d "$clone/.git" ]; then
    mkdir -p "$root/work"
    git clone --quiet "$source" "$clone" >&2
fi
if ! git -C "$clone" cat-file -e "$commit^{commit}" 2>/dev/null; then
    git -C "$clone" fetch --quiet origin >&2
fi
if [ "$(git -C "$clone" rev-parse HEAD)" != "$commit" ]; then
    git -C "$clone" checkout --quiet --detach "$commit" >&2
fi
if [ -n "$(git -C "$clone" status --porcelain)" ]; then
    echo "libraries: work/libraries has local changes; never edit the clone" >&2
    exit 1
fi
# the libraries used here (only these, so no other can shadow a name)
used=(Eigencube)
for l in "${used[@]}"; do echo "$clone/libs/$l/src"; done | paste -sd: -
