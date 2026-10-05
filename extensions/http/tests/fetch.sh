#!/usr/bin/env bash
# Serves tests/data on loopback (python3 -m http.server, a free port),
# runs tests/fetch.xtl against it, and prints what it printed (the URL as
# BASE), its exit code and the downloaded file.
#   tests/fetch.sh      (from extensions/http, xetal-x on PATH)
set -euo pipefail
dir="$(mktemp -d)"
trap '{ kill "$server"; wait "$server"; } 2>/dev/null; rm -rf "$dir"' EXIT
python3 -u -m http.server 0 --bind 127.0.0.1 --directory tests/data >"$dir/server" 2>&1 &
server=$!
port=""
for _ in $(seq 100); do
  port="$(sed -n 's/.* port \([0-9]*\).*/\1/p' "$dir/server" | head -1)"
  [ -n "$port" ] && break
  sleep 0.1
done
base="http://127.0.0.1:$port"
sed "s|\"BASE\"|\"$base\"|" tests/fetch.xtl >"$dir/fetch.xtl"
code=0
XETAL_HTTP_ROOT="$dir" xetal-x --ext . run "$dir/fetch.xtl" >"$dir/out" 2>&1 || code=$?
sed -e "s|$base|BASE|g" -e "s|$dir/||g" "$dir/out"
echo "exit: $code"
echo "-- work/quakes.csv:"
cat "$dir/work/quakes.csv"
