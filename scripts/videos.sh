#!/usr/bin/env bash
# Regenerate the demo videos: extensions/EXT/videos/NAME.webm (VP9, for
# the site) and NAME.webp (animated, inline in markdown), as X_eTaL's
# scripts/videos.sh does. Nothing appears on the screen:
#   - NAME.tape: a terminal session, rendered headlessly by vhs;
#   - NAME.sound: a demo that makes sound, run with no device, its
#     output collected in a WAV (XETAL_AUDIO_WAV): the webm is its
#     spectrogram (sox) with that sound, the webp the spectrogram;
#   - NAME.frames: a window demo run headless (XETAL_HEADLESS), every
#     frame saved (XETAL_FRAMES) and joined by ffmpeg; with `audio=FILE`
#     the file's sound goes into the webm (the visualizer: at 60 frames
#     a second a headless frame is a 60th of a second of the audio);
#   - NAME.web: a web demo served on a free port (XETAL_WEB_PORT=0, its
#     SQLite files in a fresh directory), driven by the spec's steps()
#     with `shot PATH` (the page as headless Chrome renders it, with
#     its own temporary profile: no window) and `post PATH CURL-ARGS`;
#     the shots are the frames.
#   scripts/videos.sh [EXT]
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# X_eTaL's clone (work/xetal) must exist: this repo's crates build on it
"$root/scripts/xetal.sh" >/dev/null
cd "$root"
for tool in vhs ffmpeg gif2webp sox curl; do command -v "$tool" >/dev/null || { echo "$tool not found" >&2; exit 127; }; done
# headless Chrome renders the web demos (.web): CHROME, else the usual places
chrome="${CHROME:-}"
for c in "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" google-chrome chromium; do
  [ -n "$chrome" ] && break
  if [ -x "$c" ] || command -v "$c" >/dev/null; then chrome="$c"; fi
done
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
    demo=""; seed=1; frames=120; fps=15; scale=480; audio=""; wav=""
    # shellcheck disable=SC1090
    . "$spec"
    echo "==> frames $ext $name ($frames, headless)"
    out="$root/target/screens/$ext-$name-frames"
    rm -rf "$out" && mkdir -p "$out"
    events="$(printf 'frame,%.0s' $(seq "$frames"))"
    # wav=1: the demo makes its own sound (a voice); collect it as the soundtrack
    if [ -n "$wav" ]; then audio="$root/target/screens/$ext-$name.wav"; fi
    (cd "extensions/$ext" && XETAL_HEADLESS=1 XETAL_EVENTS="$events" XETAL_FRAMES="$out" XETAL_AUDIO_WAV="$root/target/screens/$ext-$name.wav" \
      "$root/target/debug/xetal-x" --ext "$root/extensions" run --seed "$seed" "demos/$demo.xtl" > /dev/null)
    first="$(ls "$out" | head -1)"; prefix="${first%-1.png}"
    sound=()
    case "$audio" in /*) track="$audio" ;; *) track="extensions/$ext/$audio" ;; esac
    [ -n "$audio" ] && sound=(-i "$track" -map 0:v -map 1:a -c:a aac -shortest)
    ffmpeg -loglevel error -y -framerate "$fps" -i "$out/$prefix-%d.png" ${sound[@]+"${sound[@]}"} \
      -vf "scale=$scale:-1:flags=neighbor" -pix_fmt yuv420p "target/screens/$ext-$name.mp4"
    # the site's copy keeps the frame rate (and the sound); the webp is silent
    encode "target/screens/$ext-$name.mp4" "$dir/$name" "$scale" "$scale" "$([ -n "$audio" ] && echo "$fps" || echo 8)" "$audio"
  done
  for spec in "$dir"/*.web; do
    [ -e "$spec" ] || continue
    name="$(basename "$spec" .web)"
    demo=""; seed=1; size=760,560; fps=2; scale=640
    # shellcheck disable=SC1090
    . "$spec"
    echo "==> web $ext $name (headless Chrome)"
    work="$(mktemp -d)"
    out="$root/target/screens/$ext-$name-shots"
    rm -rf "$out" && mkdir -p "$out"
    (cd "extensions/$ext" && XETAL_WEB_PORT=0 XETAL_SQLITE_ROOT="$work" \
      exec "$root/target/debug/xetal-x" --ext "$root/extensions" run --seed "$seed" "demos/$demo.xtl" > "$work/out" 2>&1) &
    server=$!
    port=""
    for _ in $(seq 100); do
      port="$(head -1 "$work/out" | sed -n 's/.*127\.0\.0\.1:\([0-9]*\).*/\1/p')"
      [ -n "$port" ] && break
      sleep 0.1
    done
    [ -n "$port" ] || { cat "$work/out" >&2; exit 1; }
    shots=0
    # the page as headless Chrome draws it; Chrome lingers after
    # writing the file, so it is stopped once the file is there
    shot() {
      shots=$((shots + 1))
      local png="$out/shot-$shots.png"
      "$chrome" --headless=new --disable-gpu --no-first-run --no-default-browser-check \
        --user-data-dir="$work/chrome" --hide-scrollbars --window-size="$size" \
        --virtual-time-budget=600 --screenshot="$png" "http://127.0.0.1:$port$1" > /dev/null 2>&1 &
      local c=$!
      for _ in $(seq 200); do [ -s "$png" ] && break; sleep 0.1; done
      sleep 0.3
      kill "$c" 2>/dev/null || true
      pkill -f "$work/chrome" 2>/dev/null || true
      wait "$c" 2>/dev/null || true
    }
    post() { curl -s -o /dev/null "${@:2}" "http://127.0.0.1:$port$1"; }
    steps
    curl -s -o /dev/null "http://127.0.0.1:$port/quit"
    wait "$server" || true
    rm -rf "$work"
    ffmpeg -loglevel error -y -framerate "$fps" -i "$out/shot-%d.png" \
      -vf "scale=trunc(iw/2)*2:trunc(ih/2)*2" -pix_fmt yuv420p "target/screens/$ext-$name.mp4"
    encode "target/screens/$ext-$name.mp4" "$dir/$name" "$scale" "$scale" "$fps"
  done
  for spec in "$dir"/*.sound; do
    [ -e "$spec" ] || continue
    name="$(basename "$spec" .sound)"
    demo=""; seed=1
    # shellcheck disable=SC1090
    . "$spec"
    echo "==> sound $ext $name (no device)"
    wav="$root/target/screens/$ext-$name.wav"
    (cd "extensions/$ext" && XETAL_AUDIO=off XETAL_AUDIO_WAV="$wav" \
      "$root/target/debug/xetal-x" --ext "$root/extensions" run --seed "$seed" "demos/$demo.xtl" > /dev/null)
    png="$root/target/screens/$ext-$name-spectrogram.png"
    sox "$wav" -n remix 1,2 spectrogram -x 880 -y 360 -z 80 -t "$ext/$demo.xtl: every sample computed in X_eTaL" -o "$png"
    ffmpeg -loglevel error -y -loop 1 -framerate 2 -i "$png" -i "$wav" -c:v libvpx-vp9 -crf 40 -b:v 0 \
      -c:a libopus -b:a 64k -shortest -pix_fmt yuv420p "$dir/$name.webm"
    magick "$png" -resize 760x "$dir/$name.webp"
    ls -l "$dir/$name.webm" "$dir/$name.webp" | awk '{ print $9, $5 }'
  done
done
