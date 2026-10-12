#!/usr/bin/env bash
# The known-good xetal (XETAL_COMMIT), built by scripts/xetal.sh; prints
# its path (bin/xetal) for the scripts that run it.
#   scripts/build-xetal.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
"$root/scripts/xetal.sh" >&2
echo "$root/bin/xetal"
