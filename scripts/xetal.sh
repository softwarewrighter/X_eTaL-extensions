#!/usr/bin/env bash
# Get xetal: clone X_eTaL into work/xetal (gitignored), check out the
# known-good commit in XETAL_COMMIT, build the release binary, and
# symlink bin/xetal to it (../X_eTaL/docs/vendoring.md). Safe to run
# again: with nothing to do it only confirms the build. This repository's
# crates (xetal-x, the bridge, the page crates) build on that clone's
# crates by path, so this runs before any cargo build here.
#   scripts/xetal.sh
#   XETAL_SOURCE=../X_eTaL scripts/xetal.sh   # clone from a local checkout
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
commit="$(tr -d '[:space:]' < "$root/XETAL_COMMIT")"
source="${XETAL_SOURCE:-https://github.com/softwarewrighter/X_eTaL.git}"
clone="$root/work/xetal"

if [ ! -d "$clone/.git" ]; then
    mkdir -p "$root/work"
    git clone --quiet "$source" "$clone"
fi
if ! git -C "$clone" cat-file -e "$commit^{commit}" 2>/dev/null; then
    git -C "$clone" fetch --quiet origin
fi
if [ "$(git -C "$clone" rev-parse HEAD)" != "$commit" ]; then
    git -C "$clone" checkout --quiet --detach "$commit"
fi

(cd "$clone/components/cli" && cargo build --quiet --release -p xetal-cli >&2)

mkdir -p "$root/bin"
ln -sfn "../work/xetal/target/release/xetal" "$root/bin/xetal"
