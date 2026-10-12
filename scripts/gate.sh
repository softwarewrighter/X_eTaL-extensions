#!/usr/bin/env bash
# The pre-commit gate. By default it checks what the changes since
# origin/main call for (scripts/affected.py, as X_eTaL's gate does): the
# Rust packages and the extensions' goldens they touch, and the
# extensions whose programs use those; everything when shared code
# changed (the crates, the Cargo files, the pinned commits, the gate's
# own scripts). The cheap checks always run: the known-good X_eTaL
# (scripts/check-vendor.sh), formatting, docs/status.md, the site up to
# date (scripts/check-pages.sh), every export documented, the literate
# documents, American spellings and ASCII-only markdown. --full checks
# everything: before a release, after a batch of changes, or when in
# doubt. The machine must be free first (scripts/check-busy.sh: no
# Blender render, no other cargo). Each step's time is printed, and the
# slowest three at the end.
#   scripts/gate.sh          # what the changes call for
#   scripts/gate.sh --full   # everything
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
cd "$root"
mode=affected
[ "${1:-}" = --full ] && mode=full
started=$SECONDS
since=$SECONDS
last=""
times=""
# a step: the one before it timed
step() {
  if [ -n "$last" ]; then
    if [ $((SECONDS - since)) -ge 5 ]; then printf '    (%ss)\n' $((SECONDS - since)); fi
    times="$times$((SECONDS - since)) $last\n"
  fi
  since=$SECONDS
  last="$1"
  echo "==> $1"
}
step "machine"
"$root/scripts/check-busy.sh"
"$root/scripts/affected.py" --self-test
if [ "$mode" = full ]; then
  exts=all; rust=all; xetalx=yes; web=yes
else
  plan="$("$root/scripts/affected.py")"
  echo "$plan" | sed 's/^/    /'
  exts="$(sed -n 's/^exts: *//p' <<< "$plan")"
  rust="$(sed -n 's/^rust: *//p' <<< "$plan")"
  xetalx="$(sed -n 's/^xetal-x: *//p' <<< "$plan")"
  web="$(sed -n 's/^web: *//p' <<< "$plan")"
fi
step "X_eTaL at XETAL_COMMIT"
"$root/scripts/check-vendor.sh"
step "Rust ($rust)"
case "$rust" in
  all) "$root/scripts/check-rust.sh" ;;
  "") "$root/scripts/check-rust.sh" --none ;;
  # shellcheck disable=SC2086
  *) "$root/scripts/check-rust.sh" $rust ;;
esac
if [ "$xetalx" = yes ]; then
  step "xetal-x against the X_eTaL CLI"
  "$root/scripts/check-xetal-x.sh"
fi
step "goldens (${exts:-none})"
case "$exts" in
  all) "$root/scripts/test-exts.sh" ;;
  "") echo "test-exts: no extension changed" ;;
  # shellcheck disable=SC2086
  *) "$root/scripts/test-exts.sh" $exts ;;
esac
if [ "$web" = yes ]; then
  step "the live pages' crates"
  "$root/scripts/check-web.sh"
fi
step "status, pages, docs, literate"
"$root/scripts/status.py" --check
"$root/scripts/check-pages.sh"
"$root/scripts/check-docs.py"
"$root/scripts/literate.sh" --check
step "spelling and markdown"
"$root/scripts/check-spelling.py" --self-test
"$root/scripts/check-spelling.py"
md=(README.md CHANGES.md docs/plan.md docs/xetal-asks.md)
for f in docs/*.md extensions/*/docs/*.md web/*/README.md; do [ -e "$f" ] && md+=("$f"); done
for f in "${md[@]}"; do sw-markdown-checker -f "$f" >/dev/null || { sw-markdown-checker -f "$f"; exit 1; }; done
step "done"
printf 'slowest steps:\n'
printf "$times" | sort -n -r | head -3 | sed 's/^/    /'
echo "gate: ok ($mode, $((SECONDS - started))s)"
