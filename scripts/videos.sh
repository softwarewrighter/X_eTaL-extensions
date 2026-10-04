#!/usr/bin/env bash
# Regenerate the demo videos: extensions/EXT/videos/NAME.webm (VP9, for
# the site) and NAME.webp (animated, inline in markdown), as X_eTaL's
# scripts/videos.sh does. Nothing appears on the screen:
#   - NAME.tape: a terminal session, rendered headlessly by vhs;
#   - NAME.frames: a window demo run headless (XETAL_HEADLESS), every
#     frame saved (XETAL_FRAMES) and joined by ffmpeg; with `audio=FILE`
#     the file's sound goes into the webm (the visualizer: at 60 frames
#     a second a headless frame is a 60th of a second of the audio).
#   scripts/videos.sh [EXT]
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
for tool in vhs ffmpeg gif2webp; do command -v "$tool" >/dev/null || { echo "$tool not found" >&2; exit 127; }; done
cargo build -q --workspace
mkdir -p target/screens
encode() { # SRC DEST-BASE SCALE WEBP-SCALE [FPS [AUDIO]]
  local audio=(-an)
  [ -n "${6:-}" ] && audio=(-c:a libopus -b:a 64k)
  ffmpeg -loglevel error -y -i "$1" -vf "fps=${5:-8},scale=$3:-1:flags=neighbor" \
    -c:v libvpx-vp9 -crf 40 -b:v 0 -row-mt 1 "${audio[@]}" "$2.webm"
  ffmpeg -loglevel error -y -i "$1" -vf "fps=6,scale=$4:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=32[p];[b][p]paletteuse=dither=none" \
    "target/screens/$(basename "$2").gif"
  gif2webp -lossy -q 35 -m 4 "target/screens/$(basename "$2").gif" -o "$2.webp" > /dev/null
  ls -l "$2.webm" "$2.webp" | awk '{ print $9, $5 }'
}
for dir in extensions/${1:-*}/videos; do
  [ -d "$dir" ] || continue
  ext="$(basename "$(dirname "$dir")")"
  for tape in "$dir"/*.tape; do
    [ -e "$tape" ] || continue
    name="$(basename "$tape" .tape)"
    echo "==> vhs $ext $name"
    vhs "$tape" > /dev/null
    encode "target/screens/$ext-$name.mp4" "$dir/$name" 880 760
  done
  for spec in "$dir"/*.frames; do
    [ -e "$spec" ] || continue
    name="$(basename "$spec" .frames)"
    demo=""; seed=1; frames=120; fps=15; scale=480; audio=""
    # shellcheck disable=SC1090
    . "$spec"
    echo "==> frames $ext $name ($frames, headless)"
    out="$root/target/screens/$ext-$name-frames"
    rm -rf "$out" && mkdir -p "$out"
    events="$(printf 'frame,%.0s' $(seq "$frames"))"
    (cd "extensions/$ext" && XETAL_HEADLESS=1 XETAL_EVENTS="$events" XETAL_FRAMES="$out" \
      "$root/target/debug/xetal-x" --ext "$root/extensions" run --seed "$seed" "demos/$demo.xtl" > /dev/null)
    first="$(ls "$out" | head -1)"; prefix="${first%-1.png}"
    sound=()
    [ -n "$audio" ] && sound=(-i "extensions/$ext/$audio" -map 0:v -map 1:a -c:a aac -shortest)
    ffmpeg -loglevel error -y -framerate "$fps" -i "$out/$prefix-%d.png" ${sound[@]+"${sound[@]}"} \
      -vf "scale=$scale:-1:flags=neighbor" -pix_fmt yuv420p "target/screens/$ext-$name.mp4"
    # the site's copy keeps the frame rate (and the sound); the webp is silent
    encode "target/screens/$ext-$name.mp4" "$dir/$name" "$scale" "$scale" "$([ -n "$audio" ] && echo "$fps" || echo 8)" "$audio"
  done
done
