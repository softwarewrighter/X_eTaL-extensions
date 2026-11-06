#!/usr/bin/env bash
# Build the pinned GPU tool outside pristine clones. Its own evaluator
# dependency stays at work/gpu/XETAL_COMMIT; the animation evaluator and
# root XETAL_COMMIT are unchanged. No device execution occurs here.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
"$root/scripts/libraries.sh" >/dev/null
"$root/scripts/xetal.sh" >/dev/null
tool="$root/extensions/blender/work/gpu-tool"
pin="$(cat "$root/work/gpu/XETAL_COMMIT")"
mkdir -p "$tool/work"
if [ ! -d "$tool/work/xetal/.git" ]; then
    git clone --quiet --no-hardlinks --no-checkout "$root/work/xetal" "$tool/work/xetal"
    git -C "$tool/work/xetal" checkout --quiet --detach "$pin"
fi
[ "$(git -C "$tool/work/xetal" rev-parse HEAD)" = "$pin" ] || { echo "gpu-video: evaluator dependency pin differs" >&2; exit 1; }
[ -z "$(git -C "$tool/work/xetal" status --porcelain)" ] || { echo "gpu-video: evaluator dependency is dirty" >&2; exit 1; }
"$root/.venv/bin/python3" - "$root/work/gpu/components" "$tool/components" <<'PY'
import shutil
import sys
shutil.copytree(sys.argv[1], sys.argv[2], dirs_exist_ok=True, ignore=shutil.ignore_patterns("target"))
PY
export CARGO_TARGET_DIR="$root/work/gpu-video-target"
cargo build -q --release --locked --manifest-path "$tool/work/xetal/components/cli/Cargo.toml" -p xetal-cli
cargo build -q --release --locked --manifest-path "$tool/components/Cargo.toml" -p xetal-gpu-cli
cargo build -q --release --locked --manifest-path "$root/extensions/blender/tools/gpu-core/Cargo.toml"
printf '%s\n' "$CARGO_TARGET_DIR/release/xetal-gpu"
