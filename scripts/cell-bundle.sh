#!/usr/bin/env bash
# A render bundle of a cell program (decision C11): everything a render
# node needs with only Blender -- the scene (Cycles.xtl's player, the
# cell player, the simulator's calls), the models, job.toml (frames,
# fps, resolution, samples, engine, the Blender it was made for, a sha256
# a file), render.sh, encode.sh, README.txt -- in
# extensions/cell/work/cell/bundles/NAME/ and NAME.zip. No repository,
# Rust or X_eTaL on the node.
#   scripts/cell-bundle.sh NAME [SAMPLES]      (just ext cell bundle NAME)
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$root/.venv/bin:$PATH" VIRTUAL_ENV="$root/.venv"
"$root/scripts/venv.sh" --check
name="${1:?usage: cell-bundle.sh NAME [SAMPLES]}"
samples="${2:-64}"
cell="$root/extensions/cell"
prog="$cell/programs/$name.cell"
[ -f "$prog" ] || { echo "cell-bundle: no program $prog" >&2; exit 1; }
models="$cell/work/cell/models"
[ -f "$models/arm.glb" ] || { echo "cell-bundle: no models yet: just ext cell models" >&2; exit 1; }
out="$cell/work/cell/bundles/$name"
mkdir -p "$out/models"
(cd "$root" && cargo build -q --workspace)
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
text="$(python3 -c 'import sys; print(open(sys.argv[1]).read().replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n"), end="")' "$prog")"
# the simulator's calls for the program, and Cycles.xtl's player
# (Blender.xtl writes only under the working directory: the root's)
rel="extensions/cell/work/cell/bundles/$name"
cat > "$work/bundle.xtl" <<XTL
"bl:" u_se< "Blender"
"cy:" u_se< "Cycles"
"ce:" u_se< "Cell"
_ := "$rel/calls.js" bl:w_rite! "$name" ce:s_cene "$text"
_ := "extensions/cell/work/cell/bundles/.head.py" bl:w_rite! cy:head
XTL
(cd "$root" && XETAL_PATH="$cell/lib:$root/extensions/blender/lib" target/debug/xetal-x --ext extensions run "$work/bundle.xtl" > /dev/null)
cp "$models"/*.glb "$out/models/"
cp "$root/extensions/blender/tools/farm-range.py" "$out/render.py"
blender_version="$({ "${BLENDER:-blender}" --version 2>/dev/null || /Applications/Blender.app/Contents/MacOS/Blender --version 2>/dev/null; } | head -1 | awk '{print $2}')"
python3 - "$out" "$name" "$samples" "$cell/work/cell/bundles/.head.py" "$cell/models/cell_player.py" "${blender_version:-unknown}" <<'PY'
import hashlib, os, re, sys
out, name, samples, head, player, version = sys.argv[1:7]
fps, w, h = 30, 1280, 720
calls = open(os.path.join(out, "calls.js"), encoding="utf-8").read()
ends = [float(m.group(1)) for m in re.finditer(r"^step\([^,]+, '(?:[^'\\]|\\.)*', [0-9.e-]+, ([0-9.e-]+)\)$", calls, re.M)]
seconds = max(ends) if ends else 1.0
last = 1 + round(seconds * fps)
scene = (open(head, encoding="utf-8").read()
         + "\n# ---- the cell player (extensions/cell/models/cell_player.py) ----\n"
         + open(player, encoding="utf-8").read()
         + "\n# ---- this bundle ----\n"
         + "HERE = os.path.dirname(os.path.abspath(__file__))\n"
         + f"stage({w}, {h}, {fps}, {seconds}, {samples}, os.environ.get('CELL_DEVICE', 'GPU'))\n"
         + "cell_view(close=True)\n"
         + "cell_load(os.path.join(HERE, 'calls.js'), os.path.join(HERE, 'models'))\n")
open(os.path.join(out, "scene.py"), "w", encoding="utf-8").write(scene)
open(os.path.join(out, "render.sh"), "w").write("""#!/bin/sh
# Render this bundle with Blender alone (a farm node's share, or all):
#   ./render.sh FIRST LAST [DEVICE]       frames into out/frames/NNNN.png
#   ./render.sh still SECONDS FILE [DEVICE]   one picture into out/FILE
# DEVICE: GPU (the default: the card if Blender finds one) or CPU.
# A frame already written is skipped, so a run that stopped is run
# again. BLENDER names Blender if it is not on PATH; CELL_OUT moves out/.
# A log in out/render.log.
here="$(cd "$(dirname "$0")" && pwd)"
out="${CELL_OUT:-$here/out}"
mkdir -p "$out"
if [ "$1" = still ]; then what="still $3 $2"; dev="${4:-GPU}"; else what="frames $1 $2"; dev="${3:-GPU}"; fi
CELL_DEVICE="$dev" "${BLENDER:-blender}" --background -noaudio --factory-startup --python-exit-code 1 \\
  --python "$here/render.py" -- "$out" "$here/scene.py" $what 2>&1 | tee -a "$out/render.log"
""")
open(os.path.join(out, "encode.sh"), "w").write(f"""#!/bin/sh
# The frames in out/frames joined into out/{name}.mp4, by Blender itself.
here="$(cd "$(dirname "$0")" && pwd)"
out="${{CELL_OUT:-$here/out}}"
"${{BLENDER:-blender}}" --background -noaudio --factory-startup --python-exit-code 1 \\
  --python "$here/render.py" -- "$out" "$here/scene.py" encode {name}.mp4 2>&1 | tee -a "$out/render.log"
""")
for f in ("render.sh", "encode.sh"):
    os.chmod(os.path.join(out, f), 0o755)
open(os.path.join(out, "README.txt"), "w").write(f"""{name}: a render bundle of the assembly cell's program {name}
(X_eTaL-extensions, the cell extension). It needs only Blender
{version} or newer (Cycles, the glTF importer it ships with).

  ./render.sh still 30 look.png        one picture, 30 s in
  ./render.sh 1 {last}                  every frame ({last}, {fps} a second)
  ./render.sh 1 100 CPU                frames 1 to 100 on the processor
  ./encode.sh                          the frames into out/{name}.mp4

scene.py is Cycles.xtl's player and the cell player, posing the models
(models/*.glb) at each frame from the simulator's calls (calls.js);
render.py runs it and renders what it is asked. job.toml says the
frames, the settings and a sha256 of each file.
""")
files = sorted(f for f in os.listdir(out) if os.path.isfile(os.path.join(out, f)) and f != "job.toml")
files += sorted("models/" + f for f in os.listdir(os.path.join(out, "models")))
sums = {f: hashlib.sha256(open(os.path.join(out, f), "rb").read()).hexdigest() for f in files}
job = [f'name = "{name}"', "first = 1", f"last = {last}", f"fps = {fps}", f"seconds = {seconds}",
       f"resolution = [{w}, {h}]", f"samples = {samples}", 'engine = "CYCLES"', f'blender = "{version}"',
       "", "[sha256]"] + [f'"{f}" = "{v}"' for f, v in sums.items()]
open(os.path.join(out, "job.toml"), "w").write("\n".join(job) + "\n")
print(f"cell-bundle: {name}: {last} frames ({seconds:.1f} s at {fps}), {samples} samples, {len(files) + 1} files")
PY
# the zip brought up to date in place (-FS: what is gone from the
# directory goes from the zip), never deleted and written again
(cd "$cell/work/cell/bundles" && zip -qr -FS "$name.zip" "$name" -x "$name/out/*")
echo "cell-bundle: extensions/cell/work/cell/bundles/$name/ and $name.zip ($(du -h "$out/../$name.zip" | cut -f1))"
