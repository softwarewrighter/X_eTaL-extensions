#!/usr/bin/env bash
# Run the literate documents (docs/literate/*.org) as X_eTaL does
# (../X_eTaL/scripts/literate.sh, used here from the clone at
# XETAL_COMMIT): put the drawn form of every xetal block above it (the
# clone's scripts/literate-draw.py, by `xetal render`), then run the
# blocks with org-babel in a batch Emacs (the clone's docs/emacs) and
# record their results in the file. Blocks marked `:eval no` are listings
# of the extensions' code, not run; the `# from: FILE` line above each
# one names the file it must appear in, word for word, so a document
# cannot drift from the code it shows. --check runs on a copy and fails
# when anything recorded or drawn is out of date (commit what
# `scripts/literate.sh` changes).
#   scripts/literate.sh [--check] [DOC.org...]
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
"$root/scripts/xetal.sh" >/dev/null
clone="$root/work/xetal"
emacs="${EMACS:-}"
[ -n "$emacs" ] || ! command -v emacs > /dev/null 2>&1 || emacs=emacs
[ -n "$emacs" ] || [ ! -x /Applications/Emacs.app/Contents/MacOS/Emacs ] || emacs=/Applications/Emacs.app/Contents/MacOS/Emacs
if [ -z "$emacs" ]; then echo "literate: no Emacs; skipped"; exit 0; fi
export XETAL_BIN="$root/bin/xetal"
check=""
if [ "${1:-}" = "--check" ]; then check="--check"; shift; fi
docs=("$@")
[ ${#docs[@]} -gt 0 ] || docs=(docs/literate/*.org)
status=0
for doc in "${docs[@]}"; do
  # every listing still appears, word for word, in the file it names
  python3 - "$doc" <<'PY' || status=1
import re, sys
doc = sys.argv[1]
text = open(doc).read()
bad = 0
for m in re.finditer(r"^# from: (\S+)\n#\+begin_src xetal[^\n]*\n(.*?)^#\+end_src", text, re.S | re.M):
    path, code = m.group(1), m.group(2).rstrip("\n")
    if code not in open(path).read():
        print(f"literate: {doc}: a listing no longer matches {path}:\n{code.splitlines()[0]} ...")
        bad = 1
sys.exit(bad)
PY
  target="$doc"
  tmp=""
  if [ -n "$check" ]; then
    tmp="$(mktemp -d)"
    target="$tmp/$(basename "$doc")"
    cp "$doc" "$target"
  fi
  python3 "$clone/scripts/literate-draw.py" "$target"
  (cd "$(dirname "$target")" && "$emacs" --batch -Q -L "$clone/docs/emacs" \
      -l "$clone/docs/emacs/test/literate-run.el" "$(basename "$target")" > /dev/null 2>&1) \
    || { echo "literate: $doc failed to run"; status=1; continue; }
  if [ -n "$check" ]; then
    diff -u "$doc" "$target" || { echo "literate: $doc is out of date (run scripts/literate.sh and commit)"; status=1; }
    rm -rf "$tmp"
  else
    echo "$doc"
  fi
done
exit "$status"
