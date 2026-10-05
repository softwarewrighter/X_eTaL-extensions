#!/usr/bin/env bash
# The known-good xetal (XETAL_COMMIT), built by scripts/xetal.sh; prints
# its path (bin/xetal) for the scripts that run it.
#   scripts/build-xetal.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
"$root/scripts/xetal.sh" >&2
echo "$root/bin/xetal"
