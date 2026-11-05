# blender

Blender 3D in batch, driven from X_eTaL. An X_eTaL program writes the
Python -- a launcher that runs Blender in the background, and the `bpy`
script Blender runs inside -- runs the launcher, and reads back what it
made: rendered pictures and videos, 3D files. The chain is X_eTaL, this
extension, `python3`, Blender, Python inside Blender.

- Package: `extensions/blender/` (`extension.toml`; library
  `xetal_ext_blender`: `libxetal_ext_blender.dylib` on macOS, `.so` on
  Linux)
- Facade: `lib/Blender.xtl`, recommended alias `bl:`
- Native crates: none beyond the SDK
- Needs: `python3` (or `XETAL_PYTHON`) and Blender -- `BLENDER`, else
  `/Applications/Blender.app` on macOS, else `blender` on the PATH
  (tested with Blender 5.1)

## Functions

| Export | Type | What |
| ------ | ---- | ---- |
| `path bl:w_rite! text` | `Char -> Char -> Int` | the text into the file (made whole, its directory made); how many bytes |
| `script bl:r_un! args` | `Char -> Char -> Char` | `python3 script args` from the working directory, `XETAL_BLENDER` set to the Blender found, stopped after `XETAL_BLENDER_TIMEOUT` seconds (900 by default); the whole output in the script's `.log`; answers `exit N` (or `timeout after S s`) and the log's last 20 lines |
| `bl:r_ead path` | `Char -> Char` | a file's text (a report a run wrote) |
| `bl:f_iles dir` | `Char -> Char` | the files under dir, `bytes path` a line |
| `bl:b_lender @` | `Unit -> Char` | the Blender found, or "" |
| `bl:v_ersion @` | `Unit -> Char` | its version line, or "" |
| `bl:a_bout @` | `Unit -> Char` | the extension's name and version |

Paths are relative and stay under the working directory (or
`XETAL_BLENDER_ROOT`): no absolute paths, no `..`.

## From X_eTaL

```
"bl:" u_se< "Blender"
n := "work/run.py" bl:w_rite! launcher      # the Python, as text
n := "work/scene.py" bl:w_rite! inside
"work/run.py" bl:r_un! "work/scene.py work/out"
bl:f_iles "work/out"
```

The launcher is ordinary Python: it reads `XETAL_BLENDER` and runs
`Blender --background --factory-startup --python SCRIPT -- ARGS`; the
factory settings keep a run from depending on the user's own Blender
setup, and `--python-exit-code 1` makes a Python error inside Blender
the run's failure.

Audio is disabled. Optional `XETAL_BLENDER_ARGS` adds command-line
arguments without invoking a shell. On Linux NVIDIA, use
`XETAL_BLENDER_ARGS="--gpu-backend vulkan"` if the default OpenGL
backend selects Mesa software rendering.

## Writing bpy: Bpy.xtl

`lib/Bpy.xtl` (alias `bp:`) writes Blender's Python from X_eTaL: each
function returns a piece of a script as text, and a program joins the
pieces into the script Blender runs. The geometry is X_eTaL's -- a set
of cubes is worked out as arrays, every corner and face, and written
into the script as numbers, one mesh for them all. Written for Blender
5.1 (EEVEE is `BLENDER_EEVEE`; video output is `media_type` VIDEO).

| Export | What it writes |
| ------ | -------------- |
| `bp:launcher` | the outer script: runs Blender in the background on a script |
| `bp:head` | the inner script's start: an empty scene, the output directory, helpers |
| `(centers, size) bp:c_ubeMesh 0` | (not Python) the corners and faces of n cubes, as arrays |
| `(name, centers, size, rgb) bp:c_ubes 0` | n cubes as one mesh object, colored |
| `(text, place, size, rgb) bp:l_abel 0` | 3D text standing upright |
| `(sky, strength) bp:l_ights 0` | the world's color and a sun |
| `(center, radius, height, frames) bp:o_rbit 0` | a camera turning once round the center, linear, looping |
| `(width, height, fps) bp:s_etup 0` | EEVEE, the picture's size, the frame rate |
| `bp:s_amples n` | how many samples EEVEE takes of each pixel (64 by default): a test's tiny frames need one |
| `bp:s_till file`, `bp:v_ideo file` | a PNG of the frame; every frame as an MP4 (H.264) |
| `bp:e_xports name` | `.glb` (glTF), `.obj` (with `.mtl`) and `.blend` |
| `bp:p_iece name` | a piece: an empty at its base's middle that the parts after it hang from, so it exports as one |
| `(name, at, turn) bp:g_roup 0` | a group in the piece: an empty the parts after it hang from (placed from its middle) until `bp:ungroup` -- what moves in a clip (a crossbow, a banner, a turret) turns about its own pivot |
| `bp:ungroup` | the parts after it hang from the piece again |
| `(name, at, size, turn, rgb, emit) bp:b_ox 0` | a box (size: width depth height) |
| `... bp:c_ylinder 0` | a 12-sided cylinder (size: radius radius height) |
| `... bp:c_one 0` | a cone or frustum (size: bottom radius, top radius, height) |
| `... bp:s_phere 0` | a sphere, 12 segments and 8 rings (size: radii) |
| `... bp:t_orus 0` | a ring lying flat (size: radius, tube radius) |
| `... bp:o_ctahedron 0` | an octahedron, a crystal: corners and faces worked out in X_eTaL (`bp:o_ctahedronMesh`) |
| `(name, at, corners, faces, turn, rgb, emit) bp:m_esh 0` | a part of any shape from X_eTaL arrays (corners n by 3, faces rows of corner numbers), made to face outward: a wedge |
| `(name, at, profile, segments, turn, rgb, emit) bp:l_athe 0` | a profile (points r z) turned round its upright axis -- a dome, a pedestal -- worked out as arrays (`bp:l_atheMesh`) |
| `(part, clip, what, times, values) bp:k_eys 0` | keys a part (or the piece) in a named clip: what is `"location"`, `"rotation"` (degrees) or `"scale"`, times in seconds, a row of x y z per time; the part's rest is kept |
| `bp:e_xportPiece name` | one piece and its parts as `NAME.glb`, each clip one glTF animation by its name |
| `bp:c_lips name` | reads `NAME.glb` back and writes `NAME-clips.txt`: each clip, its first and last key, what it moves |
| `bp:report` | a report: each mesh's vertices and faces, each piece's parts and triangles, the objects |

Clips are what a game plays by name -- idle, walk, fire, hit. Each part
keyed in a clip gets an action; at export each action becomes a track
(NLA) named for its clip, and the exporter writes one glTF animation per
name with every part's keys under it. Keys are linear and land on
frames, since the exporter samples frames: at 30 frames a second (the
script's default) a tenth of a second is 3 frames, so 2 s, 1 s, 0.4 s
and 0.3 s are all exact. Part names must be unique within a script.

Every part is placed (`at`), turned (`turn`, degrees about x y z),
colored (`rgb`) and may glow (`emit`, the strength of its emission; 0
for none).

## Arrays to video: Video.xtl

`lib/Video.xtl` (alias `vd:`) turns X_eTaL arrays into an animated
scene and a video. A program works out every object's place, size and
color, the seconds it is shown, its keys, the camera's path and the
captions as arrays; each function writes one table into the script as
one call of the player -- `vd:head`, fixed Python at the top of every
script that reads the numbers, makes the objects and keys, and decides
nothing. No Python is written for a video: the same arrays always give
the same script. The look: flat colors that glow on a near-black sky,
the Standard view, JuliaMono Black (`just font`) for words, captions on
a dark plate.

| Export | What it writes |
| ------ | -------------- |
| `vd:head` | the player: an empty scene, the output directory, the functions below |
| `(w, h, fps, seconds, sky, lens) vd:s_tage 0` | EEVEE, the picture's size, the frame rate, the length, the sky, a camera that looks at an aim |
| `(name, table) vd:b_oxes 0` | a group of glowing boxes, a row each: middle, size, color, glow |
| `vd:o_utlines`, `vd:g_lass` | the same as edges only, or see-through |
| `(name, table) vd:l_ines 0` | a group of glowing lines: from, to, color, radius |
| `(name, texts, table) vd:l_abels 0` | a group of words facing the camera (texts joined by `\|`; `vd:f_ixed` writes numbers so) |
| `(texts, table) vd:c_aptions 0` | captions on a dark plate, each with its span and color, sized to the picture and wrapped to its width |
| `(titles, subtitles, table) vd:c_ards 0` | title cards filling the picture, each with its span and colors |
| `(name, spans) vd:s_hown 0` | when each object of a group is shown |
| `(name, times, places) vd:m_oves 0` | each object keyed to a place and size at the times (n by k by 6) |
| `(name, times, places) vd:m_ovesEach 0` | the same, each object at times of its own (n by k) |
| `(name, times, colors) vd:t_ints 0` | each object's color keyed at the times |
| `(name, times, strengths) vd:g_lows 0` | how strongly each object glows, keyed from data at times of its own (n by k) |
| `(name, what, rows) vd:s_teps 0` | keys that hold until the next (a lamp, a marker, a bar's length), a row each: object, second, value -- a signal sampled once a frame |
| `(name, place, texts, times) vd:r_eadout 0` | words in front of the camera that change as the video plays (a clock count) |
| `(name, path, place) vd:p_icture 0` | a picture (PNG) in the scene, its own colors: a screenshot beside the model |
| `(times, eyes, aims) vd:c_amera 0` | the camera's keys |
| `(name, origin, u, v, colors, order, gap) vd:g_rid 0` | an n by m grid of cells as one mesh, each appearing at its second: a matrix as a wall (262,144 cells for 512 by 512) |
| `(values, low, high) vd:t_one 0` | a color for each value, deep blue to orange (not Python) |
| `vd:l_inear`, `vd:c_ontrast`, `vd:f_ixed` | colors made linear; WCAG contrast (7 or more for words); numbers as labels (not Python) |
| `(times, name) vd:s_tills 0`, `vd:r_ender file`, `vd:report` | stills, the video (MP4, H.264), a report of the groups |
| `vd:landscape`, `vd:shorts`, `(width, height, lens, w, h) vd:f_it 0` | 1920 by 1080 and 1080 by 1920; how far back a camera stands for a region to fill either (not Python) |
| `(map, times) vd:r_etime 0` | a scene's own seconds as the video's: slow motion and time-lapse (not Python) |
| `(texts, table) vd:c_ues 0`, `(texts, table) vd:s_rt 0` | the captions as a cue sheet (TOML) and as subtitles (.srt), to record narration against (not Python) |

The videos' arrays come from X_eTaL-gpu's Accel library (the functions
its GPU runs), pinned at `GPU_COMMIT` and put on `XETAL_PATH` by
`scripts/libraries.sh`. Each video is explained block by block, from
the arrays to the MP4, in a literate document:
[ReLU and softmax](https://softwarewrighter.github.io/X_eTaL-extensions/literate/relu-softmax.html)
and [matrix multiply](https://softwarewrighter.github.io/X_eTaL-extensions/literate/matmul.html).

For explainer videos (YouTube, narrated later), the same scene is made
as a video (`vd:landscape`) or a Short (`vd:shorts`): `vd:f_it` places
the camera so a region fills either, and captions and title cards are
sized to the picture's shorter side and wrapped to its width. A scene
is worked out in its own seconds and every time in it passed through
`vd:r_etime`, a map from the scene's seconds to the video's, so one
part can be slow motion and another a time-lapse. `vd:c_ues` and
`vd:s_rt` write the cue sheet and the subtitles of the captions, the
times to record narration against. Words stay few and large: shapes
and data carry the explanation, not listings. The FPGA data comes from
X_eTaL-fpga, pinned at `FPGA_COMMIT` like X_eTaL-gpu (Hex1000, with
Radix and Strings, on `XETAL_PATH`). A simulation's waveform drives a
video directly: `demos/Vcd.xtl` (`vc:`) reads a VCD file into arrays
(`vc:r_ead`, `vc:c_hanges`, and `vc:a_t`, any signal's value at any
times), so every lamp is a signal sampled once a frame and keyed with
`vd:s_teps`.

## Demos

1. `demos/hello.xtl` (`just demo blender hello`): the whole chain once.
   The program writes the launcher and a script that writes the names
   of the objects in Blender's starting scene to a report, runs the
   launcher, and reads the report back: `objects: Camera Cube Light`.
   Recorded as `videos/hello.webm`.
2. `demos/cubes.xtl` (`just demo blender cubes`): how big is a thousand?
   One cube, a 10 by 10 slab of a hundred, a 10 by 10 by 10 block of a
   thousand, side by side and labeled (xkcd style), the camera going
   once round them in 10 seconds (240 frames, 640 by 360, a seamless
   loop). Every cube's corners and faces are X_eTaL arrays written into
   the script (`demos/Sizes.xtl`, `lib/Bpy.xtl`); Blender renders it
   with EEVEE -- the Standard view, thin outlines on every edge
   (Freestyle), labels turned to face the camera -- and writes the scene
   as files to open elsewhere: `sizes.glb` (glTF), `sizes.obj` (with
   `.mtl`) and `sizes.blend`, all in `work/cubes/out/`. The demo renders
   from scratch each time: about five minutes (the outlines take most
   of it). `just ext blender orbit` plays the video already rendered.
   Its recording is Blender's own render (`videos/cubes.film`). (Until
   2026-10-11 the outlines failed silently -- an empty scene's line set
   has no line style -- so the first video showed only the gaps between
   cubes; `bl:r_un!` now counts the error lines in Blender's log, and
   the tests print that count.)
3. `demos/td-models.xtl` (`just demo blender td-models`): the twelve
   tower defense pieces of X_eTaL-games -- arrow, cannon and frost towers
   at three levels, runner, brute and shield creeps -- described in
   X_eTaL (`demos/Td.xtl`) and written by Blender as
   `work/td/out/NAME.glb`, about two seconds for all twelve. Each stands
   on a 1 by 1 tile at z 0, facing -y (-z in glTF), a tower about 0.8
   wide and a creep about 0.5, in flat colors, the frost crystals
   glowing; the largest has 1228 triangles. Each has its named clips:
   `idle` (2 s, a loop) for all, `fire` (0.4 s) for towers, `walk` (1
   s, a loop) and `hit` (0.3 s) for creeps. The parts that move hang in
   groups and the clips are keyed from each part's rest. The demo reads
   every file's clips back and prints them.

4. `demos/game-assets.xtl` (`just demo blender game-assets`, about five
   minutes or more): game assets for
   [X_eTaL-games' tower defense](https://softwarewrighter.github.io/X_eTaL-games/tower-defense/),
   made by X_eTaL -> Python -> Blender -> Python. The program writes the
   launcher and one bpy script; Blender makes the twelve pieces of
   `demos/Td.xtl`, writes each as `NAME.glb` with its clips, renders
   each alone as `NAME.png` (320 by 320, transparent, all at one scale)
   and as two sheets for a game without 3D, joined in Blender:
   `NAME-turn.png` (16 frames of 256 in a 4 by 4 sheet, the piece
   turned 22.5 degrees counterclockwise each frame from its front, the
   camera 20 degrees up) and `NAME-map.png` (8 frames of 96 in a row,
   the piece facing east, then each 45 degrees clockwise on screen, an
   orthographic camera 30 degrees up as the game's 2:1 board has it: a
   tile 64 by 32 pixels, its middle at (48, 82) of a frame). Each piece
   is also written at rest as `NAME-mesh.toml`, a low-poly mesh for a
   renderer written in X_eTaL, which reads TOML (`bp:m_eshToml`):
   top-level lists of strings, `vertices` ("x y z", meters, Z up, front
   -y), `faces` (three vertex numbers, from 1) and `colors` (each
   face's `#rrggbb`, sRGB), at most 500 triangles -- the cannons, over
   that, are decimated (collapsed) until they fit. X_eTaL reads one
   back with `[]L_IST` and `n_umbers` (`tests/mesh-read.xtl`). It renders
   two videos: the twelve one after another, close and
   framed to each, turning once round and playing each clip, captioned
   (`closeups.mp4`, a minute), and the twelve lined up -- turning, idle,
   then the towers firing and the creeps walking (`lineup.mp4`, 8
   seconds). Then the program hashes every file with the digest
   extension into `manifest.toml` (the pieces, files, meshes, SHA-256s, clips,
   license, conventions, program and commit) and writes `index.html`,
   a page that shows each piece in a 3D viewer
   ([model-viewer](https://modelviewer.dev/)) with its clips on buttons,
   all in `work/td/site/`. `just ext blender publish-td` copies them to
   the site, stamping the manifest with the commit:
   [blender/td/](https://softwarewrighter.github.io/X_eTaL-extensions/blender/td/).
   The recordings show only what Blender made: `videos/game-assets.film`
   the twelve preview pictures side by side, then the lineup;
   `videos/td-models.film` (the demo above) the close-ups.

   A note on directions: each piece faces -y in Blender, which the glTF
   exporter turns into +z, glTF's own front (y up). In the lineup video
   (not in the files) each part's rest pose sits on a track under the
   clips: Blender's NLA puts a property no playing strip keys at zero,
   not at its rest.

5. `demos/relu-softmax.xtl` (`just demo blender relu-softmax`, a few
   minutes): ReLU and softmax on X_eTaL-gpu's tiny network, 30 seconds.
   z1's twenty values stand as bars on a glass floor; the nine negatives
   turn red and shrink to 0, leaving outlines. z2's four rows of two
   scores go through softmax a step at a time -- less the row's
   largest, e to each, the row's sum, divided -- and each row ends as
   a column of two shares, 1 tall (`demos/ReluSoftmax.xtl`;
   `work/explain/out/relu-softmax.mp4` and five stills).
6. `demos/matmul-steps.xtl` (`just demo blender matmul-steps`, a few
   minutes): C = A x B for X_eTaL-gpu's matmul demo ([3 4] x [4 2]), 15.5
   seconds: A on the left wall, B on the ceiling, C on the back wall;
   item by item the row and column light up, their four products appear
   in the cube with guide lines, fly into C and become its item; then
   all six at once, a work-item each (`demos/Matmul.xtl`).
7. `demos/matmul-fly.xtl` (`just demo blender matmul-fly`, about ten
   minutes, most of it X_eTaL working out the walls): the same for 512
   by 512 matrices as a fly-through, 18 seconds: the whole cube, one
   work-group's 16 by 16 tile, one work-item's 512 products and their
   running sum, then C filling in a work-group at a time
   (`demos/MatmulFly.xtl`).
8. `demos/cosy.xtl` (`just demo blender cosy`, several minutes: 120
   frames in Cycles): the CoSy logo in 3D, 4 seconds turning once
   round, looping. C, o, S and y are round magenta tubes, toothpaste
   style, melted together where they touch, on yellow. Each letter is
   its center line, an array of points traced from the logo and placed
   in meters by X_eTaL (`demos/CosyLogo.xtl`); Blender gives each line a
   round tube (a curve beveled round) and a sphere on each end, joins
   them, remeshes the union in voxels and smooths the seams, then
   renders glossy plastic with Cycles (`work/cosy/out/cosy.mp4`). On
   the site the demo links [CoSy](https://cosy.com) and
   [reva-fork](https://github.com/softwarewrighter/reva-fork). Cycles
   runs on the GPU with Metal when there is one, its kernels
   unoptimized: Blender 5.1.2 crashed optimizing them in the background
   partway through a render.
9. `demos/hex-pc.xtl` (`just demo blender hex-pc`, a few minutes: 586
   frames, twice): HEX1000's program counter, 19.5 seconds, made with
   the explainer kit. The 64 addresses of a ROM page are cells in
   reading order; the program counter visits them in its shift
   register's order -- 0 1 3 7 15 31 63 62 ..., never +1 -- and each
   cell flashes as it is reached and stays lit in a color for when, the
   path drawn behind it. The order is X_eTaL-fpga's `hx:Order`; a title
   card, the first six hops in slow motion and the other 57 as a
   time-lapse, captions. Made as a video (`work/hexpc/out/hex-pc.mp4`,
   1920 by 1080) and a Short (`work/hexpc/out-short/hex-pc-short.mp4`,
   1080 by 1920), with its cue sheet and subtitles
   (`work/hexpc/hex-pc.cues.toml`, `hex-pc.srt`; `demos/PcWalk.xtl`).
10. `demos/hex-run.xtl` (`just ext blender hex-run`, about 25 minutes:
    1261 frames, twice): HEX1000-UART computing C0DE+BEEF=, 42 seconds,
    from a simulation of its board. `scripts/hex1000-sim.sh` builds
    X_eTaL-fpga's board (its top, its UART core, the machine's
    Verilog, at `FPGA_COMMIT`) and runs it under iverilog
    (`demos/hex-run.tb.v`), the text sent over the serial line bit by
    bit, into a waveform; `scripts/hex1000-browser.py` types the same
    text at the live browser calculator in headless Chrome (its
    DevTools pipe, no window) and saves its screen. The program reads
    the waveform and checks it before writing anything: the line
    carries the text, the board's UART receives it, Hex1000.xtl
    replaying the clocks from the K inputs the circuit saw has the
    circuit's A, S, PC and PA at every one (4971 clocks), and the
    display ends as the evaluator's `hx:c_alc` and the browser's
    (17FCD). Then the video (`demos/HexRun.xtl`): four areas -- the
    serial line (each character's ten bits lit as they arrive, the
    bytes received), the key matrix (the column lamps R8 to R13, the
    row lamps K, the held key, the board's queue), the machine (the ROM
    page with the program counter's marker, the page, A, the status,
    the adder, the RAM's 64 words as bars) and a calculator's display
    (its segments lit from the digit latches, the browser's screen
    under it) -- every lamp a signal sampled once a frame; a clock
    count, and while slow the instruction, in front of the camera. The
    first character and the key matrix finding it in slow motion, the
    other keys as a time-lapse, the addition (AMAAC, a digit at a time
    with its carry) in slow motion, a title card, captions. A video
    (`work/hexrun/out/hex-run.mp4`) and a Short
    (`work/hexrun/out-short/hex-run-short.mp4`), with its cue sheet and
    subtitles (`work/hexrun/hex-run.cues.toml`, `hex-run.srt`).

11. `demos/hex-build.xtl` (`just ext blender hex-build`, with the OSS
    CAD Suite on `PATH`): the HEX1000-UART build process, from `hx:s_tep` through
    hand-lowered XIR and emitted Verilog, Yosys synthesis, nextpnr
    placement and routing, and `gowin_pack`'s bitstream entering a
    schematic Tang Nano 9K. Blue LUTs and orange flip-flops move from
    a cloud to their actual BEL coordinates. Green marks the tiles
    occupied by routed wires; the view fits the occupied region, in
    tile coordinates, without claiming physical wire paths or lengths.
    The board silhouette is schematic, without measured dimensions.
    The animation represents configuration; the recipe builds the
    bitstream without accessing hardware.

    `scripts/hex1000-build.py` runs the pinned FPGA board's tool flow
    outside its clone and retains `synth.json`, `pnr.json`,
    `hex1000.fs`, logs, and `evidence.json` (artifact hashes, counts,
    cell names and BELs) in `work/hexbuild/`. Its small numeric tables
    drive `demos/HexBuild.xtl`; counts in the captions come from the
    build. Tool versions can change synthesis and packing: this
    machine's Yosys produced 647 LUTs and 469 flip-flops, plus memory,
    carry and wide multiplexer cells; nextpnr packing produced 1350
    placed LUT/flip-flop cells. Neither count is hard-coded.

    Outputs: `work/hexbuild/out/hex-build.mp4` (1920 by 1080),
    `out-short/hex-build-short.mp4` (1080 by 1920), both 24 seconds
    at 30 fps; `hex-build.cues.toml` and `hex-build.srt` beside them.
    `just ext blender hex-build-data` rebuilds just the evidence;
    `just demo blender hex-build` renders existing evidence.
    `just ext blender hex-build-check` tests the evidence reader,
    including missing placement and malformed or absent routing.
    `tests/hex-build-quick.xtl` renders four stages in both shapes
    from an actual build; `blender-hex-build-quick` uses a small,
    stable fixture with LUTs and flip-flops sharing a tile.

## HEX1000 video families

**HEX1000-UART** is the Tang Nano 9K design with serial input. Its
execution and build-process recordings keep their `hex-run` and
`hex-build` commands, file paths and site anchors. **HEX1000-keys-leds**
is the separate Tang Nano 1K project with a physical 4x4 keypad, four
operator buttons and an eight-digit MAX7219 display. Its execution video
is `hex-keys-leds-run`; its build-process video is `hex-keys-leds-build`. The
program-counter video (`hex-pc`) is shared HEX1000 material.

| Family | Recording | Duration | Export directory |
| ------ | --------- | -------- | ---------------- |
| HEX1000-UART (Nano 9K) | [execution](https://softwarewrighter.github.io/X_eTaL-extensions/#blender-hex-run) | 42 seconds | `work/hexrun/` |
| HEX1000-UART (Nano 9K) | [build](https://softwarewrighter.github.io/X_eTaL-extensions/#blender-hex-build) | 24 seconds | `work/hexbuild/` |
| HEX1000-keys-leds (Nano 1K) | [execution](https://softwarewrighter.github.io/X_eTaL-extensions/#blender-hex-keys-leds-run) | 36 seconds | `work/keysrun/` |
| HEX1000-keys-leds (Nano 1K) | [build](https://softwarewrighter.github.io/X_eTaL-extensions/#blender-hex-keys-leds-build) | 32 seconds | `work/keysbuild/` |

Each export directory contains a 1920 by 1080 landscape master under
`out/`, a 1080 by 1920 Short under `out-short/`, and its cue sheet and
SRT. Both shapes use 30 fps. Durations above are the nominal timelines;
the inclusive final frame adds 1/30 second to each file. The site uses
1280 by 720 recordings at 30 fps, with the existing public URLs intact.

After rendering and recording, run `just ext blender hex-release-check`
with ffmpeg and ffprobe on PATH.
It decodes and counts all twelve video streams (eight exports and four
recordings), checks cue/SRT text and timing, verifies site copies and
family headings, and extracts critical stages from both shapes. Its
hashes, pinned commits and sampled frames stay in `work/hex-release/`.
The report records media-tool versions and verifies the saved build,
execution and scene-array hashes. Synthesis/router/packer versions,
executable and device-database hashes are retained in the physical
calculator's build evidence below.
The original UART execution master and Short were absent at the final
audit; they were regenerated from the pinned board simulation and a
fresh headless browser calculation, without changing their public URL.
The UART film recipes allow one hour per Blender run by default; set
`XETAL_BLENDER_TIMEOUT` to override it. The execution script stops on a
failed render. Linux NVIDIA renders can select Vulkan with
`XETAL_BLENDER_ARGS='--gpu-backend vulkan'`.

`just ext blender uart-titles` refreshes the static opening cards and
cue sheets using the existing checked build counts and a fresh UART
simulation. It renders six title stills with X_eTaL/Blender, replaces
only the introductory frames, and re-encodes the two site recordings.
Original inputs and stream/hash evidence stay in `work/uart-titles/`.
Repeated retitling reuses those original inputs; use the full demo
renderer when the footage itself changes.
Existing full masters are refreshed at their own resolutions; missing
masters are left for the full demo renderer, which uses the new titles.
The recording specifications accept optional `HEX_RUN_VIDEO` and
`HEX_BUILD_VIDEO` source paths for this operation. Keep the OSS CAD
Suite and recording tools on PATH, and build the counts first with
`just ext blender hex-build-data` if needed.

## Physical 1K execution video

`just ext blender hex-keys-leds-run` simulates the complete pinned board,
checks it against the physical project's Hex1000 and Front models, then
renders a 36-second landscape video and vertical Short, both at 30 fps.
The OSS CAD Suite must be on PATH. Linux NVIDIA users can select Vulkan
with `XETAL_BLENDER_ARGS='--gpu-backend vulkan'`.

External matrix contacts and operator buttons enter `C0DE+BEEF=`. The
production startup and 256-clock row timing are retained, with a nominal
27 MHz stimulus (37.038 ns per simulated period). All 221,161 active
clocks agree with the X_eTaL core/controller models: registers, R,
scanner, debounce, handshake, digits and SPI state. Independently decoded
DIN/SCLK/LOAD transactions (6,143 words) show `DEADBEEF`, `17FCD`, then
zero after the second equals. A short bounce is rejected; held keys
produce one accepted event each.

X_eTaL chooses the scene geometry and time map. The waveform adapter
samples the checked VCD at those times; every contact, scan lamp,
accepted lamp, serial pin and LED segment comes from that waveform.
Captions identify slow motion and time-lapse, including a single serial
word expanded for inspection. Board geometry is schematic, using the
Panel library's key positions rather than measured dimensions. This is
simulation evidence; no hardware is accessed.

Masters, cue sheet, subtitles, trace, waveforms, hashes and model logs
remain in `work/keysrun/`; `just videos blender hex-keys-leds-run` makes
the site recording from the existing master. `hex-keys-leds-preview`
renders six critical stills in both shapes from checked evidence;
`hex-keys-leds-wave-check` tests malformed serial transactions. The replay
uses bounded chunks to avoid scalar selection copying a whole large
trace (ask E11 in `docs/xetal-asks.md`).

## Physical 1K build-process video

`just ext blender hex-keys-leds-build` builds the full pinned calculator
with the OSS CAD Suite, verifies its evidence, then renders a 32-second
landscape video and vertical Short at 30 fps. The input is the physical
calculator with keypad, operators, MAX7219 and TXS0108E OE, using the
same independent source pin as the execution video.

The opening identifies Hex1000 and Front specifications, the core's
hand-lowered XIR and the hand-written controller/wrapper RTL. This
project does not contain the upstream compiler. Yosys, nextpnr and
gowin_pack supply the build evidence; the video shows packed resource
counts, including carry, block RAM and I/O, rather than treating every
placed cell as a LUT. The 20 pad BELs include 16 ordinary I/O buffers
and four tri-state row outputs, which nextpnr reports separately.

All 1,382 cell identities and BEL assignments are retained. Colors
distinguish LUTs, flip-flops, carry ALUs, block RAM, I/O and other cells.
Cell positions come from their actual BEL tiles; offsets within tiles
keep the schematic symbols visible. The 1,422 routed nets occupy 206
unique wire tiles. Green marks that footprint, without claiming
physical wire geometry. Captions report both passing clocks and the
divider-feedback dedicated-route warning, with BUFG routing to the
machine clock sinks.

The closing configuration animation shows the 679,890-byte packed file
entering a schematic Nano 1K beside its keypad, operator strip, level
shifter and eight-digit display. Geometry uses scene units; this is
build/pack evidence with no hardware detection or programming.

`hex-keys-leds-build-data` verifies existing artifacts and exports their
scene arrays into `work/keysbuild/`, with source-evidence and array
hashes plus the complete cell identity/BEL list in `scene-evidence.json`.
It checks recorded counts and timing against the router report and
bitstream size against the packed file. `hex-keys-leds-build-preview`
renders six stages in both shapes from those artifacts;
`hex-keys-leds-build-check` tests incomplete placement, identities,
resource totals, routing and timing. The regression uses a small fixed
fixture containing all six resource classes. Masters, cues and SRT stay
in `work/keysbuild/`; `just videos blender hex-keys-leds-build` encodes
the existing landscape master for the site.

## Physical 1K calculator evidence

`HEX1000_COMMIT` independently pins
[hardwarewrighter/hex1000](https://github.com/hardwarewrighter/hex1000)
at `b7148fea99121e60f4f58151682dce96cbbfa5be`. `just hex1000` clones it
into `work/hex1000` and its own pinned Strings dependency into
`work/hex1000-libraries`. Both clones must remain clean. Their libraries
are isolated from the existing UART demos; `FPGA_COMMIT` is unchanged.
The physical project's evaluator pin is checked against `XETAL_COMMIT`.

With the OSS CAD Suite on PATH:

```sh
just hex1000
just ext blender hex-keys-leds-check
just ext blender hex-keys-leds-evidence
just ext blender hex-keys-leds-verify
```

The selected top is the full `hex1000_calculator` with keypad, operator
strip and MAX7219, using the TXS0108E OE output on FPGA23. It retains
explicit `TO=SIG` row-enable polarity, the global machine clock, input
synchronizers and production startup/debounce counts. The adapter
follows the pinned project's `scripts/board-calculator.py`, copies its
RTL/testbenches into extension-owned output directories and never runs
the clone's build or programming scripts.

`work/keys-leds-evidence/evidence.json` records source/transitive pins,
input and artifact hashes, exact commands, tool versions/fingerprints,
device databases, simulation results and resource/timing evidence.
`synth.json`, `pnr.json`, `timing.json`, `cells.json`, `routes.json`,
`calculator.fs` and logs remain beside it. Verification rejects changed
pins, input copies and artifacts. A new run invalidates the previous
success record before building. `hex-keys-leds-sim` runs only the model
comparison and board simulations, in `work/keys-leds-sim`.

The Front model matches 1,100 RTL clocks. Complete calculator simulations
at 4 and 256 clocks per row cover the startup banner, rejected bounce,
first key, held-key handshake, arithmetic, second-equals clear and
MAX7219 initialization/framing/segments. The upstream testbench uses a
100 MHz stimulus clock; its production divider, row and startup counts
are retained. Board timing targets the actual 27 MHz input and
27/28 MHz machine clock. This build uses:

| Resource | Used | Available |
| -------- | ---- | --------- |
| LUTs | 772 | 1,152 |
| Flip-flops | 289 | 864 |
| Carry ALUs | 174 | 864 |
| Block RAMs | 2 | 4 |
| I/O buffers (router report) | 16 | 48 |

Four additional tri-state row outputs occupy four distinct pad BELs;
the full calculator uses 20 pads. nextpnr lists TBUF separately with no
standalone capacity; its 16-IOB count is not the physical pin count.

Routed maximum frequencies are 148.721 MHz for the 27 MHz input and
71.937 MHz for the 0.964286 MHz machine clock; both pass. The one
dedicated-route warning is the divider's data-feedback inverter; clock
sinks use the checked machine BUFG. The uncompressed bitstream is
679,890 bytes. Counts and timing come from the saved router report.

These are fresh simulation/build results. The source project's physical
arithmetic and flash-CRC observations are dated historical evidence.
No device detection, SRAM loading or flash programming is performed.
Overflow still wraps, and upstream still lists overflow text, ROM-side
RAM clearing, complete physical key coverage and a separate cold boot
as follow-ups. The execution video uses the fresh simulation described
above.

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (rust/tests) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
```

The Rust tests run a stand-in program in Blender's place; the goldens
`blender-demo-hello`, `blender-bpy-tiny`, `blender-cubes-quick` and
`blender-cosy-quick` run Blender itself (in a temporary directory;
`blender-cubes-quick` renders the size comparison at 160 by 90, four
frames, and checks 8, 800 and 8,000 vertices and the files;
`blender-cosy-quick` fuses the CoSy logo in coarse voxels and checks
the one mesh left, its pivot, camera and lights, and a still;
`blender-hex-pc-quick` makes HEX1000's program counter at 160 by 90
and as a Short, 90 by 160, and checks its cue sheet and subtitles and
what Blender made of each; `blender-hex-run-quick` simulates the board
(it needs iverilog), checks the waveform as the demo does and makes
the HEX1000 run at 160 by 90 and 90 by 160).
`blender-keys-leds-quick` simulates the physical 1K with production
timing, replays all active clocks and decodes the serial display, then
renders six stages at 640 by 360 and 360 by 640.
Of them: the second writes two cubes and a label as bpy
and checks what Blender made -- the mesh (16 vertices, 12 faces), the
objects, a still, `.glb`, `.obj`, `.mtl`, `.blend`. `blender-bpy-script`
pins the Python the library writes. `blender-shapes` makes one piece of
every kind of part, `blender-clips` a piece with two clips read back
from its `.glb`, and `blender-td-models` the twelve pieces (their
triangles, their clips, the files).
