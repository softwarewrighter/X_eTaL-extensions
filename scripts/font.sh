#!/usr/bin/env bash
# The videos' font: JuliaMono (https://github.com/cormullion/juliamono,
# SIL Open Font License 1.1), a monospace font with APL and mathematical
# symbols, so code shows as written. Pinned at a release, each file
# checked by its SHA-256, into work/fonts (gitignored; Blender scripts
# look there first, then the user's installed fonts).
#   scripts/font.sh            # work/fonts/JuliaMono-{Regular,Medium,Black}.ttf and LICENSE
#   scripts/font.sh --check    # fail unless they are there and right
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release=v0.63.2
url="https://github.com/cormullion/juliamono/releases/download/$release/JuliaMono-ttf.tar.gz"
tarball=be6517295198ec5c92bdbaad42f4f6f8d83f921d80512b79f54fe036add95c0c
files="JuliaMono-Regular.ttf:40a07da0d1601215eb6b89312eb44128a3e2f36675d3e1f518264bd391fc7023
JuliaMono-Medium.ttf:5ba89520d89ac7ff2e646391dc3a78b56177ddc6e8ceb7cdb06a9f5f9edb2544
JuliaMono-Black.ttf:02c4fab01eaae8753b403c4ec2ba2b970fd8559fa29a08f7e9a0ccbff6a14e59"
dir="$root/work/fonts"
sha() { shasum -a 256 "$1" 2>/dev/null | cut -d' ' -f1; }
ok() { for f in $files; do [ "$(sha "$dir/${f%%:*}")" = "${f#*:}" ] || return 1; done; [ -f "$dir/LICENSE" ]; }
if [ "${1:-}" = --check ]; then
  ok || { echo "font: JuliaMono $release is not in work/fonts (run: just font)" >&2; exit 1; }
  exit 0
fi
if ok; then echo "font: JuliaMono $release in work/fonts"; exit 0; fi
mkdir -p "$dir"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
curl -fsSL -o "$tmp/fonts.tar.gz" "$url"
[ "$(sha "$tmp/fonts.tar.gz")" = "$tarball" ] || { echo "font: $url is not the pinned file" >&2; exit 1; }
tar xzf "$tmp/fonts.tar.gz" -C "$tmp" LICENSE JuliaMono-Regular.ttf JuliaMono-Medium.ttf JuliaMono-Black.ttf
for f in $files; do
  [ "$(sha "$tmp/${f%%:*}")" = "${f#*:}" ] || { echo "font: ${f%%:*} is not the pinned file" >&2; exit 1; }
  cp "$tmp/${f%%:*}" "$dir/"
done
cp "$tmp/LICENSE" "$dir/LICENSE"
ok && echo "font: JuliaMono $release in work/fonts"
