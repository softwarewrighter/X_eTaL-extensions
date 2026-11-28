#!/usr/bin/env bash
# An agent driving the cell over MCP: Claude Code in print mode, the
# cell's MCP server its only tools, given extensions/cell/mcp/goals/NAME.goal.
# Its session (every command the server ran) goes to
# extensions/cell/work/mcp/NAME.cell, its transcript to NAME.jsonl, and
# its last message is printed. It costs a Claude Code session.
#   scripts/cell-agent.sh NAME [MAX_TURNS]      (just ext cell agent NAME)
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
name="${1:?usage: cell-agent.sh NAME [MAX_TURNS]}"
turns="${2:-80}"
goal="$root/extensions/cell/mcp/goals/$name.goal"
[ -f "$goal" ] || { echo "cell-agent: no goal $goal" >&2; exit 1; }
command -v claude > /dev/null || { echo "cell-agent: no claude (Claude Code) on PATH" >&2; exit 1; }
(cd "$root" && cargo build -q -p xetal-cell-mcp -p xetal-x)
w="$root/extensions/cell/work/mcp"
mkdir -p "$w"
cat > "$w/$name.json" <<EOF
{"mcpServers": {"cell": {"command": "$root/target/debug/xetal-cell-mcp", "env": {"XETAL_CELL_SESSION": "$w/$name.cell", "XETAL_CELL_PORT": "0"}}}}
EOF
(cd "$w" && claude -p "$(cat "$goal")" --mcp-config "$name.json" --strict-mcp-config \
  --allowedTools mcp__cell --max-turns "$turns" --output-format stream-json --verbose \
  < /dev/null > "$name.jsonl")
tail -1 "$w/$name.jsonl" | python3 -c 'import json, sys
d = json.loads(sys.stdin.read())
print("cell-agent: %s turns, %s s, %s USD" % (d.get("num_turns"), round(d.get("duration_ms", 0) / 1000), d.get("total_cost_usd")))
print(d.get("result", ""))'
echo "cell-agent: its session: extensions/cell/work/mcp/$name.cell"
