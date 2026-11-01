#!/usr/bin/env bash
# Get the shared X_eTaL libraries used here: clone X_eTaL-libraries into
# work/libraries, X_eTaL-gpu into work/gpu and X_eTaL-fpga into work/fpga
# (all gitignored) and check out the known-good commits in
# LIBRARIES_COMMIT, GPU_COMMIT and FPGA_COMMIT, as scripts/xetal.sh does
# for X_eTaL. Prints the directories to put on
# XETAL_PATH (the src/ directories of the libraries used here,
# colon-separated): the demos that use a shared library reach it there --
# the Rubik's cube solver (Eigencube, X_eTaL-libraries), the explainer
# videos (Accel, X_eTaL-gpu's acceleratable subset: the layers the GPU
# runs; Hex1000, X_eTaL-fpga's 4-bit machine, with Radix and Strings,
# which it uses). Never edit a clone; move to a newer commit by changing its file.
#   scripts/libraries.sh
#   LIBRARIES_SOURCE=../X_eTaL-libraries GPU_SOURCE=../X_eTaL-gpu FPGA_SOURCE=~/github/hardwarewrighter/X_eTaL-fpga scripts/libraries.sh   # clone from local checkouts
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
# clone REPO-URL DIR COMMIT-FILE: DIR at the commit, unedited
pin() {
    local source="$1" clone="$2" commit
    commit="$(tr -d '[:space:]' < "$3")"
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
        echo "libraries: ${clone#"$root"/} has local changes; never edit the clone" >&2
        exit 1
    fi
}
pin "${LIBRARIES_SOURCE:-https://github.com/softwarewrighter/X_eTaL-libraries.git}" "$root/work/libraries" "$root/LIBRARIES_COMMIT"
pin "${GPU_SOURCE:-https://github.com/softwarewrighter/X_eTaL-gpu.git}" "$root/work/gpu" "$root/GPU_COMMIT"
pin "${FPGA_SOURCE:-https://github.com/hardwarewrighter/X_eTaL-fpga.git}" "$root/work/fpga" "$root/FPGA_COMMIT"
# the libraries used here (only these, so no other can shadow a name)
{
    echo "$root/work/libraries/libs/Eigencube/src"
    echo "$root/work/libraries/libs/Strings/src"
    echo "$root/work/gpu/libs/Accel/src"
    echo "$root/work/fpga/libs/Hex1000/src"
    echo "$root/work/fpga/libs/Radix/src"
} | paste -sd: -
