#!/usr/bin/env bash
# Runs tests/serve.xtl and asks it four requests with curl, on loopback;
# prints what the program printed (its port as PORT) and each answer.
#   tests/serve.sh      (from extensions/web, xetal-x on PATH)
set -euo pipefail
out="$(mktemp)"
trap 'rm -f "$out"' EXIT
xetal-x --ext . run tests/serve.xtl >"$out" 2>&1 &
pid=$!
port=""
for _ in $(seq 100); do
  port="$(head -1 "$out")"
  [ -n "$port" ] && break
  sleep 0.1
done
ask() { curl -s -w ' [%{http_code} %{content_type}]\n' "$@"; }
ask "http://127.0.0.1:$port/hello?name=X_eTaL%20web"
ask -d "x=1 2 3 4.5" "http://127.0.0.1:$port/sum"
ask "http://127.0.0.1:$port/square?half=3"
ask "http://127.0.0.1:$port/nowhere"
wait "$pid"
echo "-- the program printed:"
sed "s/^$port\$/PORT/" "$out"
