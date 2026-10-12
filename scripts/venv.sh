#!/usr/bin/env bash
# The repository's Python: a uv virtual environment in .venv on a
# uv-managed CPython (PYTHON, 3.12 by default), never the system's.
# The scripts use the standard library only, so nothing is installed;
# a package a script comes to need goes in with `uv pip install`.
# The justfile and the gate put .venv/bin first on PATH.
#   scripts/venv.sh            # make .venv if it is missing
#   scripts/venv.sh --check    # fail unless python3 on PATH is .venv's
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
want="${PYTHON:-3.12}"
if [ "${1:-}" = --check ]; then
  have="$(command -v python3 || true)"
  [ "$have" = "$root/.venv/bin/python3" ] || {
    echo "venv: python3 is ${have:-missing}, not $root/.venv/bin/python3 (run: just venv)" >&2
    exit 1
  }
  exit 0
fi
command -v uv >/dev/null || { echo "venv: uv not found (brew install uv)" >&2; exit 127; }
if [ -x "$root/.venv/bin/python3" ]; then
  echo "venv: $("$root/.venv/bin/python3" --version) in .venv"
  exit 0
fi
uv venv --python "$want" --python-preference only-managed "$root/.venv"
echo "venv: $("$root/.venv/bin/python3" --version) in .venv"
