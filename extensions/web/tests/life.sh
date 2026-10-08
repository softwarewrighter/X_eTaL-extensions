#!/usr/bin/env bash
# Drives demos/life.xtl over loopback with curl: the page, a glider on
# an 8 by 8 board stepped three times as SVG, a random board, a 404, and
# quit; prints each answer and what the program printed (its port as
# PORT).
#   tests/life.sh      (from extensions/web, xetal-x on PATH)
set -euo pipefail
out="$(mktemp)"
trap 'rm -f "$out"' EXIT
XETAL_WEB_PORT=0 xetal-x --ext . run demos/life.xtl >"$out" 2>&1 &
pid=$!
port=""
for _ in $(seq 100); do
  port="$(head -1 "$out" | sed -n 's/.*127\.0\.0\.1:\([0-9]*\).*/\1/p')"
  [ -n "$port" ] && break
  sleep 0.1
done
u="http://127.0.0.1:$port"
ask() { curl -s -w ' [%{http_code} %{content_type}]\n' "$@"; }
curl -s -o /dev/null -w '/ [%{http_code} %{content_type}]\n' "$u/"
curl -s "$u/" | grep -o '<h1>[^<]*</h1>\|/life.svg'
ask "$u/glider?n=8"
for _ in 1 2 3; do ask "$u/life.svg"; done
ask "$u/new?n=8"
curl -s -o /dev/null -w 'a random 8 by 8 board [%{http_code} %{content_type}]\n' "$u/life.svg"
ask "$u/nowhere"
ask "$u/quit"
wait "$pid"
echo "-- the program printed:"
sed "s/:$port\$/:PORT/" "$out"
