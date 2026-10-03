#!/usr/bin/env bash
# Start an extension from templates/extension/.
#   scripts/new-ext.sh NAME ALIAS "what it does"
# NAME is lowercase (sqlite); the facade is its capitalized form
# (Sqlite.xtl); the crate is xetal-ext-NAME, the library xetal_ext_NAME.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
name="${1:?usage: new-ext.sh NAME ALIAS \"what it does\"}"
alias="${2:?alias, e.g. sq}"
what="${3:?a one-line description}"
[[ "$name" =~ ^[a-z][a-z0-9]*$ ]] || { echo "new-ext: NAME must be lowercase letters and digits" >&2; exit 1; }
dest="$root/extensions/$name"
[ -e "$dest" ] && { echo "new-ext: $dest exists" >&2; exit 1; }
cap="$(printf '%s' "${name:0:1}" | tr '[:lower:]' '[:upper:]')${name:1}"
stem="xetal_ext_$name"
cp -R "$root/templates/extension" "$dest"
mv "$dest/lib/__CAP__.xtl" "$dest/lib/$cap.xtl"
find "$dest" -type f ! -name .gitkeep -print0 | while IFS= read -r -d '' f; do
  NAME="$name" CAP="$cap" STEM="$stem" ALIAS="$alias" WHAT="$what" perl -pi -e \
    's/__NAME__/$ENV{NAME}/g; s/__CAP__/$ENV{CAP}/g; s/__STEM__/$ENV{STEM}/g; s/__ALIAS__/$ENV{ALIAS}/g; s/__WHAT__/$ENV{WHAT}/g' "$f"
done
echo "new-ext: extensions/$name (crate xetal-ext-$name, facade lib/$cap.xtl, alias $alias:)"
