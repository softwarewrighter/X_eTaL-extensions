#!/usr/bin/env bash
# Build the live site into pages/, which is committed: the Pages
# workflow publishes that folder as it is (nothing is built on GitHub).
# As ../X_eTaL-demos does.
#   - every extension with a live page (extensions/NAME/web/) is built
#     with trunk into pages/NAME/, served under /X_eTaL-extensions/NAME/;
#   - pages/index.html, the catalog (scripts/build-catalog.py).
# An extension that loses its web/ loses its pages/NAME/.
#   scripts/build-pages.sh
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
base="/X_eTaL-extensions"
command -v trunk >/dev/null || { echo "trunk not found on PATH" >&2; exit 127; }
mkdir -p "$root/pages"
touch "$root/pages/.nojekyll"
keep=()
for web in "$root"/extensions/*/web; do
  [ -f "$web/Cargo.toml" ] || continue
  name="$(basename "$(dirname "$web")")"
  keep+=("$name")
  dist="$root/target/pages-dist/$name"
  echo "==> trunk build $name"
  (cd "$web" && trunk build --release --public-url "$base/$name/" --dist "$dist")
  rsync -a --delete "$dist/" "$root/pages/$name/"
done
for d in "$root"/pages/*/; do
  [ -d "$d" ] || continue
  s="$(basename "$d")"
  printf '%s\n' ${keep[@]+"${keep[@]}"} | grep -qx "$s" || { echo "removing pages/$s/"; rm -rf "$d"; }
done
cp "$root/images/modern-xetal-logo.jpg" "$root/images/favicon.ico" "$root/web/shell/shell.css" "$root/pages/"
"$root/scripts/build-catalog.py"
echo "pages/ built; commit it (git add pages/) and push to publish."
