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
| `(texts, table) vd:c_aptions 0` | captions on a dark plate, each with its span and color |
| `(name, spans) vd:s_hown 0` | when each object of a group is shown |
| `(name, times, places) vd:m_oves 0` | each object keyed to a place and size at the times (n by k by 6) |
| `(name, times, places) vd:m_ovesEach 0` | the same, each object at times of its own (n by k) |
| `(name, times, colors) vd:t_ints 0` | each object's color keyed at the times |
| `(times, eyes, aims) vd:c_amera 0` | the camera's keys |
| `(name, origin, u, v, colors, order, gap) vd:g_rid 0` | an n by m grid of cells as one mesh, each appearing at its second: a matrix as a wall (262,144 cells for 512 by 512) |
| `(values, low, high) vd:t_one 0` | a color for each value, deep blue to orange (not Python) |
| `vd:l_inear`, `vd:c_ontrast`, `vd:f_ixed` | colors made linear; WCAG contrast (7 or more for words); numbers as labels (not Python) |
| `(times, name) vd:s_tills 0`, `vd:r_ender file`, `vd:report` | stills, the video (MP4, H.264), a report of the groups |

The videos' arrays come from X_eTaL-gpu's Accel library (the functions
its GPU runs), pinned at `GPU_COMMIT` and put on `XETAL_PATH` by
`scripts/libraries.sh`. Each video is explained block by block, from
the arrays to the MP4, in a literate document:
[ReLU and softmax](https://softwarewrighter.github.io/X_eTaL-extensions/literate/relu-softmax.html)
and [matrix multiply](https://softwarewrighter.github.io/X_eTaL-extensions/literate/matmul.html).

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
the one mesh left, its pivot, camera and lights, and a still). Of them: the second writes two cubes and a label as bpy
and checks what Blender made -- the mesh (16 vertices, 12 faces), the
objects, a still, `.glb`, `.obj`, `.mtl`, `.blend`. `blender-bpy-script`
pins the Python the library writes. `blender-shapes` makes one piece of
every kind of part, `blender-clips` a piece with two clips read back
from its `.glb`, and `blender-td-models` the twelve pieces (their
triangles, their clips, the files).
