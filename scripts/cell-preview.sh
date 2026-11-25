#!/usr/bin/env bash
# The assembly cell's preview, the slicer: a program
# (extensions/cell/programs/NAME.cell) run by the simulator, written
# as a scene (ce:s_cene) and as a page by the three extension
# (th:p_age!) with the cell's models, into extensions/cell/work/preview/NAME;
# then served on this repository's port, 8470. programs/NAME.scene, if
# there is one, holds calls added to the scene (parts the simulator does
# not place yet, ghosts, expectations).
#   scripts/cell-preview.sh NAME          # write and serve at http://127.0.0.1:8470/
#   scripts/cell-preview.sh NAME --write  # write only
# BIND=0.0.0.0 serves it to the local network. The models come from
# `just ext cell models` (work/cell/models).
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Python is the uv environment's only, never the system's (CLAUDE.md rule 19)
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
name="${1:?usage: cell-preview.sh NAME [--write]}"
cell="$root/extensions/cell"
program="$cell/programs/$name.cell"
[ -f "$program" ] || { echo "cell-preview: no program $program" >&2; exit 1; }
models="$cell/work/cell/models"
[ -f "$models/arm.glb" ] || { echo "cell-preview: no models yet: just ext cell models" >&2; exit 1; }
(cd "$root" && cargo build -q --workspace)
out="work/preview/$name"
mkdir -p "$cell/work/preview"
# the program as X_eTaL text: backslashes, quotes and newlines escaped
text="$(python3 -c 'import sys; print(open(sys.argv[1]).read().replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n"), end="")' "$program")"
cat > "$cell/work/preview/$name.xtl" <<XTL
"ce:" u_se< "Cell"
"th:" u_se< "Three"
"$out" th:p_age! ("$name" ce:s_cene "$text")
XTL
(cd "$cell" && "$root/target/debug/xetal-x" --ext "$root/extensions" run "work/preview/$name.xtl" > /dev/null)
# NAME.scene, if there is one: calls added to the scene for the player
# (parts, ghosts, expectations), until the simulator writes them itself
extra="$cell/programs/$name.scene"
[ -f "$extra" ] && grep -v '^//' "$extra" >> "$cell/$out/scene.calls.js"
mkdir -p "$cell/$out/models"
cp "$models"/*.glb "$cell/$out/models/"
echo "cell-preview: $name written to extensions/cell/$out ($(grep -c '^step(' "$cell/$out/scene.calls.js") steps)"
[ "${2:-}" = --write ] && exit 0
bind="${BIND:-127.0.0.1}"
echo "cell-preview: serving at http://$bind:8470/ (Ctrl-C stops)"
exec python3 -m http.server 8470 --bind "$bind" -d "$cell/$out"
