#!/usr/bin/env bash
# xetal-x is the known-good xetal with an ext: store: every X_eTaL demo
# must give the same output, errors and exit code under both, and an
# ext: path must reach the extensions.
#   scripts/check-xetal-x.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
xetal="$("$root/scripts/build-xetal.sh")"
(cd "$root" && cargo build -q --workspace)
xx="$root/target/debug/xetal-x"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
n=0
for f in "$root"/work/xetal/demos/*.xtl; do
  a="$(cd "$scratch" && "$xetal" run --seed 1 "$f" 2>&1 </dev/null; echo "exit $?")"
  b="$(cd "$scratch" && "$xx" run --seed 1 "$f" 2>&1 </dev/null; echo "exit $?")"
  [ "$a" = "$b" ] || { echo "check-xetal-x: $(basename "$f") differs:"; diff <(echo "$a") <(echo "$b") | head -20; exit 1; }
  n=$((n + 1))
done
version="$("$xx" --version)"
printf '%s\n' "$version" | grep -q "$(head -c 7 "$root/XETAL_COMMIT")" \
  || { echo "check-xetal-x: --version does not name the known-good commit" >&2; exit 1; }
case "$version" in xetal-x*X_eTaL-extensions*) ;; *) echo "check-xetal-x: --version does not name xetal-x" >&2; exit 1 ;; esac
listing="$("$xx" --ext "$root/extensions" --ext-list)"
printf '%s\n' "$listing" | grep -q '^  hello/shout : Char -> Char' \
  || { echo "check-xetal-x: hello not listed" >&2; exit 1; }
echo "check-xetal-x: ok ($n demos identical)"
