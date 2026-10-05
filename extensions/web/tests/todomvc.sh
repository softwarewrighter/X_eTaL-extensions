#!/usr/bin/env bash
# Drives demos/todomvc.xtl over loopback with curl, its database in a
# fresh directory: add (an HTML-looking title, an empty one ignored),
# toggle, filter, delete, clear completed; then the program is run again
# and the list is still there. Prints each step's answer: the list's
# items (title, state) and the footer, and what the program printed.
#   tests/todomvc.sh      (from extensions/web, xetal-x on PATH)
set -euo pipefail
dir="$(mktemp -d)"
out="$dir/out"
trap 'rm -rf "$dir"' EXIT
export XETAL_SQLITE_ROOT="$dir" XETAL_WEB_PORT=0
start() {
  xetal-x --ext .. run demos/todomvc.xtl >"$out" 2>&1 &
  pid=$!
  port=""
  for _ in $(seq 100); do
    port="$(head -1 "$out" | sed -n 's/.*127\.0\.0\.1:\([0-9]*\).*/\1/p')"
    [ -n "$port" ] && break
    sleep 0.1
  done
  u="http://127.0.0.1:$port"
}
post() { curl -s -o /dev/null -w "$1 [%{http_code} -> %{redirect_url}]\n" "${@:2}" | sed "s|$u|URL|"; }
# the page, summed up: each item's state and title, then the footer's count
show() {
  echo "page$1:"
  curl -s "$u/$1" | sed 's|</li>|\n|g' \
    | sed -n 's|.*<li class=\([a-z]*\)>.*<span>\(.*\)</span>.*|  \1: \2|p'
  curl -s "$u/$1" | grep -o '<footer><span>[^<]*' | sed 's|<footer><span>|  |'
  curl -s "$u/$1" | grep -o "class=on>[a-z]*" | sed 's|class=on>|  filter: |'
}
start
post add --data-urlencode 'title=milk & <eggs>' "$u/add"
post add -d 'title=write the demo' "$u/add"
post add -d 'title=record it' "$u/add"
post "add (empty)" -d 'title=' "$u/add"
show ""
post toggle -d id=2 "$u/toggle"
show "?show=active"
show "?show=completed"
post delete -d id=3 "$u/delete"
show ""
curl -s -w ' [%{http_code}]\n' "$u/quit"
wait "$pid"
echo "-- run again: the list is kept in SQLite"
start
show ""
post clear -X POST "$u/clear"
show ""
curl -s -w ' [%{http_code}]\n' "$u/nowhere"
curl -s -w ' [%{http_code}]\n' "$u/quit"
wait "$pid"
echo "-- the program printed:"
sed "s/:$port\$/:PORT/" "$out"
