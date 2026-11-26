#!/usr/bin/env bash
# The planner (CellPlan.xtl): extensions/cell/nets/NAME.net, a netlist,
# planned into extensions/cell/programs/NAME.cell, a command program the
# simulator runs, the preview plays and the live page offers.
#   scripts/cell-plan.sh NAME
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
name="${1:?usage: cell-plan.sh NAME}"
cell="$root/extensions/cell"
net="$cell/nets/$name.net"
[ -f "$net" ] || { echo "cell-plan: no netlist $net" >&2; exit 1; }
(cd "$root" && cargo build -q --workspace)
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
# the netlist as X_eTaL text: backslashes, quotes and newlines escaped
text="$(python3 -c 'import sys; print(open(sys.argv[1]).read().replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n"), end="")' "$net")"
cat > "$work/plan.xtl" <<XTL
"cp:" u_se< "CellPlan"
cp:p_lan "$text"
XTL
(cd "$cell" && XETAL_PATH="$cell/lib" "$root/target/debug/xetal-x" --ext . run "$work/plan.xtl" > "$work/out")
cp "$work/out" "$cell/programs/$name.cell"
echo "cell-plan: $name planned into extensions/cell/programs/$name.cell ($(grep -vc '^#' "$cell/programs/$name.cell") commands)"
