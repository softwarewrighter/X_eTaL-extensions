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
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
out="${1:?usage: literate-html.sh DIR}"
cd "$root"
"$root/scripts/xetal.sh" >/dev/null
clone="$root/work/xetal"
emacs="${EMACS:-}"
[ -n "$emacs" ] || ! command -v emacs > /dev/null 2>&1 || emacs=emacs
[ -n "$emacs" ] || [ ! -x /Applications/Emacs.app/Contents/MacOS/Emacs ] || emacs=/Applications/Emacs.app/Contents/MacOS/Emacs
if [ -z "$emacs" ]; then echo "literate-html: no Emacs; skipped"; exit 0; fi
export XETAL_BIN="$root/bin/xetal"
# the libraries the documents use: the shared ones (scripts/libraries.sh:
# Accel, Eigencube) and the blender extension's pure X_eTaL libraries
# (Bpy, Video, and the demos' shared ReluSoftmax and Matmul)
XETAL_PATH="$("$root/scripts/libraries.sh"):$root/extensions/blender/lib:$root/extensions/blender/demos"
export XETAL_PATH
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
# the index: the documents grouped by their #+CATEGORY: line, in the
# order the groups first appear (Blender before Voxels: blender-*.org,
# relu-softmax.org, then water.org), and the other X_eTaL repositories'
{
  echo '<!DOCTYPE html>'
  echo '<html lang="en"><head><meta charset="utf-8"/><title>Literate documents -- X_eTaL Extensions</title>'
  echo '<link rel="stylesheet" href="style.css"/></head><body><div id="content">'
  echo '<h1 class="title">Literate documents</h1>'
  echo '<p>The extensions explained with their code, drawn as X_eTaL draws it, and the results it gives.</p>'
  for cat in $(for doc in docs/literate/*.org; do sed -n 's/^#+CATEGORY: //p' "$doc"; done | awk '!seen[$0]++' | tr ' ' '~'); do
    cat="${cat//\~/ }"
    echo "<h2>$cat</h2><ul>"
    for doc in docs/literate/*.org; do
      [ "$(sed -n 's/^#+CATEGORY: //p' "$doc")" = "$cat" ] || continue
      name="$(basename "$doc" .org)"
      title="$(sed -n 's/^#+TITLE: //p' "$doc")"
      sub="$(sed -n 's/^#+SUBTITLE: //p' "$doc")"
      echo "<li><a href=\"$name.html\">$title</a> -- $sub</li>"
    done
    echo '</ul>'
  done
  echo '<h2>Elsewhere</h2><ul>'
  echo '<li><a href="https://softwarewrighter.github.io/X_eTaL/literate/">X_eTaL&#39;s literate documents</a> -- the language itself: a beginner&#39;s start, a tour, libraries, classics</li>'
  echo '</ul><p><a href="../">The demos</a></p></div></body></html>'
} > "$out/index.html"
