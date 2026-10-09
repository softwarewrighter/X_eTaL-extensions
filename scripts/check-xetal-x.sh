#!/usr/bin/env bash
# xetal-x is the known-good xetal with an ext: store: every X_eTaL demo
# must give the same output, errors and exit code under both, and an
# ext: path must reach the extensions.
#   scripts/check-xetal-x.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
xetal="$("$root/scripts/build-xetal.sh")"
(cd "$root" && cargo build -q --release --workspace)
xx="$root/target/release/xetal-x"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
# the demos side by side, each in its own scratch directory (a demo may
# write files where it runs); both builds are release, as the CLI is
one() {
  local f="$1" d
  d="$scratch/$(basename "$f" .xtl)"
  mkdir -p "$d/a" "$d/b"
  local rc=0
  (cd "$d/a" && "$xetal" run --seed 1 "$f" > "$d/a.out" 2>&1 </dev/null) || rc=$?
  echo "exit $rc" >> "$d/a.out"
  rc=0
  (cd "$d/b" && "$xx" run --seed 1 "$f" > "$d/b.out" 2>&1 </dev/null) || rc=$?
  echo "exit $rc" >> "$d/b.out"
}
n=0
for f in "$root"/work/xetal/demos/*.xtl; do
  one "$f" &
  n=$((n + 1))
done
wait
for f in "$root"/work/xetal/demos/*.xtl; do
  d="$scratch/$(basename "$f" .xtl)"
  cmp -s "$d/a.out" "$d/b.out" || { echo "check-xetal-x: $(basename "$f") differs:"; diff "$d/a.out" "$d/b.out" | head -20; exit 1; }
done
version="$("$xx" --version)"
printf '%s\n' "$version" | grep -q "$(head -c 7 "$root/XETAL_COMMIT")" \
  || { echo "check-xetal-x: --version does not name the known-good commit" >&2; exit 1; }
case "$version" in xetal-x*X_eTaL-extensions*) ;; *) echo "check-xetal-x: --version does not name xetal-x" >&2; exit 1 ;; esac
listing="$("$xx" --ext "$root/extensions" --ext-list)"
printf '%s\n' "$listing" | grep -q '^  hello/shout : Char -> Char' \
  || { echo "check-xetal-x: hello not listed" >&2; exit 1; }
echo "check-xetal-x: ok ($n demos identical)"
