#!/usr/bin/env bash
# The live city's ride as a video, from the page itself: the scene
# written by `just ext three city` (extensions/three/work/city) is
# served on this repository's port, headless Chrome (no window) draws
# it at moments of the flight (?t=SECONDS holds the flight there), and
# ffmpeg joins the shots: extensions/three/work/city/ride.mp4.
#   scripts/three-ride.sh [FPS]     (10 by default: 150 shots)
# THREE_RIDE_URL=URL uses a page already being served.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
fps="${1:-10}"
port=8470
site="$root/extensions/three/work/city"
[ -f "$site/scene.calls.js" ] || { echo "three-ride: nothing written yet: just ext three city" >&2; exit 1; }
chrome="${CHROME:-}"
for c in "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" google-chrome chromium; do
  [ -n "$chrome" ] && break
  if [ -x "$c" ] || command -v "$c" >/dev/null; then chrome="$c"; fi
done
[ -n "$chrome" ] || { echo "three-ride: no Chrome (set CHROME)" >&2; exit 127; }
tmp="$(mktemp -d)"
# THREE_RIDE_URL: the page already served somewhere (a server of this
# repository's on the port); else it is served here
url="${THREE_RIDE_URL:-}"
if [ -z "$url" ]; then
  if curl -s -o /dev/null "http://127.0.0.1:$port/"; then
    echo "three-ride: port $port is in use: stop what serves it, or give THREE_RIDE_URL" >&2
    exit 1
  fi
  python3 -m http.server "$port" --bind 127.0.0.1 -d "$site" > /dev/null 2>&1 &
  server=$!
  trap 'kill "$server" 2>/dev/null || true; rm -rf "$tmp"' EXIT
  url="http://127.0.0.1:$port/"
  for _ in $(seq 50); do curl -s -o /dev/null "$url" && break; sleep 0.1; done
else
  trap 'rm -rf "$tmp"' EXIT
fi
# the page as headless Chrome draws it at one moment; Chrome lingers
# after writing the file, so it is stopped once the file is there
shot() {
  local i="$1" t="$2" png="$tmp/shot-$1.png"
  "$chrome" --headless=new --use-angle="${ANGLE:-metal}" --enable-gpu --ignore-gpu-blocklist \
    --no-first-run --no-default-browser-check --user-data-dir="$tmp/profile-$i.d" --hide-scrollbars \
    --window-size=1280,720 --virtual-time-budget=4000 --screenshot="$png" \
    "$url?t=$t&hud=0" > /dev/null 2>&1 &
  local c=$!
  for _ in $(seq 300); do [ -s "$png" ] && break; sleep 0.1; done
  sleep 0.3
  kill "$c" 2>/dev/null || true
  pkill -f "$tmp/profile-$i\.d" 2>/dev/null || true
  wait "$c" 2>/dev/null || true
  [ -s "$png" ] || { echo "three-ride: no picture at $t s" >&2; return 1; }
}
frames=$((15 * fps))
i=0
while [ "$i" -lt "$frames" ]; do
  # four at a time
  for _ in 1 2 3 4; do
    [ "$i" -lt "$frames" ] || break
    i=$((i + 1))
    shot "$i" "$(python3 -c "print(($i - 1) / $fps)")" &
  done
  wait
done
ffmpeg -loglevel error -y -framerate "$fps" -i "$tmp/shot-%d.png" \
  -vf "scale=trunc(iw/2)*2:trunc(ih/2)*2" -pix_fmt yuv420p "$site/ride.mp4"
echo "three-ride: $frames shots in extensions/three/work/city/ride.mp4"
