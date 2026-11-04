#!/usr/bin/env bash
# The physical 1K calculator, independently pinned from the 9K UART project.
# Print the clean clone path. --check validates without fetching/checking out.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
cd "$root"
clone="$root/work/hex1000"
commit="$(tr -d '[:space:]' < "$root/HEX1000_COMMIT")"
source="${HEX1000_SOURCE:-https://github.com/hardwarewrighter/hex1000.git}"
if [ "${1:-}" != --check ] && [ ! -d "$clone/.git" ]; then
    mkdir -p "$root/work"
    git clone --quiet "$source" "$clone" >&2
fi
[ -d "$clone/.git" ] || { echo "hex1000: missing clone; run just hex1000" >&2; exit 1; }
[ -z "$(git -C "$clone" status --porcelain)" ] || {
    echo "hex1000: work/hex1000 has local changes; never edit the clone" >&2; exit 1;
}
if [ "${1:-}" != --check ]; then
    git -C "$clone" cat-file -e "$commit^{commit}" 2>/dev/null || git -C "$clone" fetch --quiet origin >&2
    if [ "$(git -C "$clone" rev-parse HEAD)" != "$commit" ]; then
        git -C "$clone" checkout --quiet --detach "$commit" >&2
    fi
fi
[ "$(git -C "$clone" rev-parse HEAD)" = "$commit" ] || {
    echo "hex1000: clone is not at HEX1000_COMMIT" >&2; exit 1;
}
# Keep the physical project's Strings dependency at its own transitive pin.
# Do not put these libraries on the UART demos' global XETAL_PATH.
libraries="$root/work/hex1000-libraries"
libraries_commit="$(tr -d '[:space:]' < "$clone/XETAL_LIBRARIES_COMMIT")"
if [ "${1:-}" != --check ] && [ ! -d "$libraries/.git" ]; then
    git clone --quiet "${HEX1000_LIBRARIES_SOURCE:-https://github.com/softwarewrighter/X_eTaL-libraries.git}" "$libraries" >&2
fi
[ -d "$libraries/.git" ] || { echo "hex1000: missing transitive library clone" >&2; exit 1; }
[ -z "$(git -C "$libraries" status --porcelain)" ] || {
    echo "hex1000: work/hex1000-libraries has local changes; never edit the clone" >&2; exit 1;
}
if [ "${1:-}" != --check ]; then
    git -C "$libraries" cat-file -e "$libraries_commit^{commit}" 2>/dev/null || git -C "$libraries" fetch --quiet origin >&2
    if [ "$(git -C "$libraries" rev-parse HEAD)" != "$libraries_commit" ]; then
        git -C "$libraries" checkout --quiet --detach "$libraries_commit" >&2
    fi
fi
[ "$(git -C "$libraries" rev-parse HEAD)" = "$libraries_commit" ] || {
    echo "hex1000: transitive library pin mismatch" >&2; exit 1;
}
cmp -s "$clone/XETAL_COMMIT" "$root/XETAL_COMMIT" || {
    echo "hex1000: evaluator pins differ; review compatibility before using the models" >&2; exit 1;
}
printf '%s\n' "$clone"
