#!/usr/bin/env bash
# Export the literate documents (docs/literate/*.org) to HTML in DIR
# (scripts/build-pages.sh puts them in the site as literate/), as X_eTaL
# exports its own: the clone's docs/emacs/literate-export.el (at
# XETAL_COMMIT) draws every xetal block by `xetal render --html` with the
# lines as typed after it, the recorded results as they are; X_eTaL's
# style; the pictures they show; an index page. Run scripts/literate.sh
# first (it records the results).
#   scripts/literate-html.sh DIR
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: literate-html.sh DIR}"
cd "$root"
"$root/scripts/xetal.sh" >/dev/null
clone="$root/work/xetal"
emacs="${EMACS:-}"
[ -n "$emacs" ] || ! command -v emacs > /dev/null 2>&1 || emacs=emacs
[ -n "$emacs" ] || [ ! -x /Applications/Emacs.app/Contents/MacOS/Emacs ] || emacs=/Applications/Emacs.app/Contents/MacOS/Emacs
if [ -z "$emacs" ]; then echo "literate-html: no Emacs; skipped"; exit 0; fi
export XETAL_BIN="$root/bin/xetal"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/docs/literate" "$out/images"
cp docs/literate/*.org "$tmp/docs/literate/"
"$emacs" --batch -Q -l "$clone/docs/emacs/literate-export.el" \
  "$tmp"/docs/literate/*.org > "$tmp/emacs.log" 2>&1 || { cat "$tmp/emacs.log"; exit 1; }
# pictures beside the pages; the footer's links to this site and repository
for page in "$tmp"/docs/literate/*.html; do
  sed -e 's#\.\./\.\./images/#images/#g' \
      -e 's#>Live demo</a>#>The demos</a>#' \
      -e 's#https://github.com/softwarewrighter/X_eTaL"#https://github.com/softwarewrighter/X_eTaL-extensions"#' \
      "$page" > "$out/$(basename "$page")"
done
for image in $(grep -oh 'images/[A-Za-z0-9_.-]*' docs/literate/*.org | sort -u); do
  cp "$image" "$out/images/"
done
cp "$clone/docs/literate/style.css" "$out/style.css"
{
  echo '<!DOCTYPE html>'
  echo '<html lang="en"><head><meta charset="utf-8"/><title>Literate documents -- X_eTaL Extensions</title>'
  echo '<link rel="stylesheet" href="style.css"/></head><body><div id="content">'
  echo '<h1 class="title">Literate documents</h1>'
  echo '<p>The extensions explained with their code, drawn as X_eTaL draws it, and the results it gives.</p><ul>'
  for doc in docs/literate/*.org; do
    name="$(basename "$doc" .org)"
    title="$(sed -n 's/^#+TITLE: //p' "$doc")"
    sub="$(sed -n 's/^#+SUBTITLE: //p' "$doc")"
    echo "<li><a href=\"$name.html\">$title</a> -- $sub</li>"
  done
  echo '</ul><p><a href="../">The demos</a></p></div></body></html>'
} > "$out/index.html"
