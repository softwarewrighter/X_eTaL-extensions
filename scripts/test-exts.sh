#!/usr/bin/env bash
# Every extension's reg-rs tests (their Rust tests run in check-rust.sh),
# and the layout every extension must have (plan A7). The extensions run
# side by side -- each in its own directory, its work files its own --
# after one release build; each one's result is printed in order, and a
# failure shows that extension's differences.
#   scripts/test-exts.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
"$root/scripts/xetal.sh" >/dev/null
(cd "$root" && cargo build -q --release --workspace)
export REG_EXT_BUILT=1
XETAL_LIBS="$("$root/scripts/libraries.sh")"
export XETAL_LIBS
logs="$(mktemp -d)"
trap 'rm -rf "$logs"' EXIT
names=()
for dir in "$root"/extensions/*/; do
  name="$(basename "$dir")"
  for need in extension.toml justfile rust/Cargo.toml lib tests docs/README.md demos; do
    [ -e "$dir/$need" ] || { echo "test-exts: extensions/$name has no $need" >&2; exit 1; }
  done
  names+=("$name")
  ( "$root/scripts/reg-ext.sh" "$name" run > "$logs/$name.out" 2>&1; echo $? > "$logs/$name.rc" ) &
done
wait
failed=""
for name in "${names[@]}"; do
  if [ "$(cat "$logs/$name.rc")" = 0 ]; then
    echo "test-exts: $name: $(tail -1 "$logs/$name.out")"
  else
    cat "$logs/$name.out"
    failed="$failed $name"
  fi
done
if [ -n "$failed" ]; then
  for name in $failed; do "$root/scripts/reg-ext.sh" "$name" run -vv || true; done
  echo "test-exts: failed:$failed" >&2
  exit 1
fi
echo "test-exts: ok (${#names[@]} extensions)"
