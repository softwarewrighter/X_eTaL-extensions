# Voxels: a mini game in X_eTaL, built up demo by demo

An analysis of `../avoxelgame-fork` (Kyle Croarkin's voxel game in
Dyalog APL, MIT) and of how this repository could show voxel graphics
from X_eTaL: what carries over, what our native windows can and
cannot do today, measured costs, the chunk size to use, and a series
of `voxels-*` demos that build up to a small game, each one runnable,
tested and recorded before the next. Written 2026-10-07. Analysis
only: nothing here is scheduled yet.

## In short

- The APL game's central idea carries over directly: a chunk is a 3D
  array of block numbers, and the faces to draw are the solid cells
  whose neighbor in each of six directions is air -- six rotations
  compared with the original, the same idiom as Life. X_eTaL has the
  primitives (`o_-` along an axis, `r_eplicate`, `w_here`, `t_able`).
- X_eTaL keeps the world and does the game: terrain, the exposed-face
  lists, collision, picking, the rules. Rust draws: X_eTaL is a
  tree-walking interpreter and the `ext:` bridge carries arrays as
  text, so pixels must not cross the bridge every frame; faces cross
  once per chunk change and stay in Rust, patched by id, the way the
  scene extension already keeps lines.
- This repository has no SDL. The APL game uses SDL3's GPU API from
  Dyalog's FFI; here the native windows are winit and softbuffer, drawn
  on the CPU (canvas: arrays as pixels; scene: 3D lines and dots,
  retained by id, an orbit camera). That is enough: what is missing is
  filled, depth-tested faces in scene, a first-person camera, and
  key-up and mouse-motion events -- a few hundred lines of Rust, with
  no GPU and no new system libraries, and still recordable headlessly.
- Smaller chunks help, a lot: 16 by 16 by 16 instead of the APL game's
  16 by 128 by 16. Measured below: the exposed-face mask of a chunk
  costs 1.75 ms in X_eTaL at 16 by 16 by 16 and 13 ms at 16 by 128 by
  16, and an edit only remeshes its own chunk.

## What the APL game does

About 2,400 lines of Dyalog APL and 330 of C (`lse/`, the SDL glue),
holding 60 frames a second:

| Part | How (file) | Carries over? |
| ---- | ---------- | ------------- |
| chunks | a 4D array, chunks by 16 by 128 by 16, beside an inverted table of per-chunk facts (position, buffers, dirty flag) (`world.apln`) | yes: X_eTaL arrays and a table of rows |
| meshing | every possible face precomputed once; a chunk's exposed faces chosen by a Boolean mask from six rotations of the solid mask; vertices packed into 4 bytes (`world.apln`, `Copy_chunk`) | the mask, yes; the vertex packing is for the GPU and is not needed |
| culling | each chunk's bounding box against the six frustum planes, one expression over all chunks (`Draw_chunks`) | yes, in X_eTaL or in Rust |
| collision | swept bounding box per axis, a Minkowski sum against the nearest block (`player.apln`) | yes, small arrays |
| picking | a ray marched in 100 steps over 5 blocks; a proper voxel traversal exists but is unused (`Silly_hit.aplf`, `Ray_hit.aplf`) | yes: the 100-step march is one array expression |
| terrain | 3D fractal value noise, three octaves, seeded; stone, dirt, grass, sand, water, one tree a chunk (`Load_new_chunks`) | yes, smaller: 2D noise for heights is cheaper |
| streaming | 441 chunks in view, at most two disk operations a frame | not for a mini game: a fixed small world |
| saving | Dyalog component files, run-length packed | if wanted: sqlite |
| water | a negative block number; only the surface drawn, one sea level, a sine wave | the surface, yes |
| lighting | none: texture and fog only | sky light is an array scan (below) |

The author's numbers (Dyalog 20, a MacBook): all faces of a chunk
precomputed in 3.5 ms once; a chunk's exposed faces in 0.46 ms, 1.8
ms with textures. The author names lighting and flowing water as the
two things that do not fit array style: both spread from a point until
blocked, a fixpoint over the whole 3D array.

## What this repository has

- **canvas** (winit, softbuffer): a native window showing an array as
  pixels, scaled to the window; key and click events back. Life at 60
  frames a second.
- **scene** (winit, softbuffer): retained 3D polylines, segments and
  dots, each set by id and patched by id; an orbit camera turned by
  dragging, on the CPU with antialiased lines; key presses back;
  headless frames for tests and recordings (`XETAL_HEADLESS`,
  `XETAL_FRAMES`).
- **xetal-x** gives windows the main thread and runs the program on its
  own; the program pulls events (`n_ext!`) and pushes changes.
- No GPU, no filled polygons, no depth buffer, no first-person camera,
  no key-up or mouse-motion events.

## Measured: what X_eTaL and the bridge cost

On this development Mac, X_eTaL v0.1.0 (release build); `work/`
scripts, 20 repetitions averaged.

The exposed-face mask (six rotations, six comparisons, a sum), on a
rolling height field:

| Chunk | Cells | ms per chunk | Faces |
| ----- | ----- | ------------ | ----- |
| 16 by 16 by 16 | 4,096 | 1.75 | about 1,000 |
| 16 by 32 by 16 | 8,192 | 3.35 | |
| 16 by 64 by 16 | 16,384 | 6.8 | |
| 16 by 128 by 16 | 32,768 | 13.25 | about 10,000 |

About 0.4 microseconds a cell, linear. Dyalog does the whole meshing
of a 16 by 128 by 16 chunk in 0.46 ms; X_eTaL takes 13 ms for the
mask alone, so roughly 30 times slower. Fine for meshing when a chunk
changes, too slow to remesh the world every frame.

The bridge (`ext:` text, both ways, through `hx:e_cho`):

| Floats | ms there and back |
| ------ | ----------------- |
| 4,096 | 1.4 |
| 32,768 | 11.6 |
| 196,608 (a 256 by 256 color frame) | 71 |

About 0.35 microseconds a number. So: a whole frame of pixels per
frame is out (71 ms); a chunk's face list once per change is fine
(1,000 faces as 5 numbers each, about 2 ms); the camera every frame
(6 numbers) is nothing.

## The split

| X_eTaL (the program) | Rust (scene) |
| -------------------- | ------------ |
| the world: chunks as 3D arrays of block numbers | faces kept per chunk id |
| terrain (noise, heights, layers) | quads from face lists |
| exposed faces per chunk, sent when it changes | depth buffer, flat shading per face direction, fog |
| the player: velocity, gravity, collision | a first-person camera set each frame |
| picking: the ray march; breaking, placing | culling (or X_eTaL's, as a demo of it) |
| the game's rules, score, time | events: keys down and up, mouse motion, frames |

A face crosses as five numbers: x, y, z of its cell, its direction (0
to 5) and its block type; Rust makes the quad and colors it by type
and direction (no textures at first: flat colors read well and keep
recordings small).

## Chunk size: 16 by 16 by 16

Cubic chunks, stacked: a small world of 4 by 2 by 4 chunks is 64 by 32
by 64 blocks.

- **Meshing:** 1.75 ms a chunk; the whole small world (32 chunks)
  in about 60 ms at the start, and an edit remeshes one chunk (or two
  at a border) in a couple of milliseconds.
- **The bridge:** a changed chunk's faces in about 2 ms.
- **Culling:** smaller boxes leave out more; a chunk entirely solid
  or entirely air has no faces and costs nothing.
- **The border:** the APL game treats a chunk's border as air (every
  border face drawn). With small chunks there are more borders, so do
  better: pad each chunk with a one-block shell from its neighbors
  before the rotations (18 by 18 by 18, then cut back), which removes
  the hidden faces and the wrap-around in one move.
- **Lighting and water become reachable:** the author's two walls are
  fixpoints over the whole array; over a 16-cube they are small.
  Sky light is not even a fixpoint: a running "or" down each column
  (`s_\` along the vertical axis) marks every cell under a solid one
  as shaded -- one scan. Block light spreading from a torch is at
  most 15 rounds of the six-rotation step over 16-cubes, about 25 ms,
  run only when a light or a block changes. Water flowing one step a
  tick is a cellular automaton like Life over the chunks near the
  player.

Even smaller (8 by 8 by 8) would cut an edit to under half a
millisecond but multiply the number of chunks (and ids, and bridge
calls) by eight; 16 is the balance.

## The demos, piece by piece

Each is `extensions/scene/demos/voxels-NAME.xtl`, runs with `just demo
scene voxels-NAME`, has a reg-rs golden (numbers, and headless frames
where it draws) and a recording, and is shown before the next is
started. The shared X_eTaL lives in a library beside them,
`Voxels.xtl` -- no demo may be named `voxels.xtl` (ask E8).

| # | Demo | Shows | New native work |
| - | ---- | ----- | --------------- |
| 1 | `voxels-chunk` | a 16-cube as a 3D array: a height field made into layers (stone, dirt, grass), water below a level; counts per block type; slices drawn with `[]G_RID` | none |
| 2 | `voxels-faces` | the exposed-face mask by six rotations (the Life idiom), padded with neighbors; the face list as an n by 5 array; face counts for solid, hollow and checkerboard chunks; the faces drawn as wireframe squares in scene, orbiting | none (scene segments) |
| 3 | `voxels-solid` | the same chunk drawn solid: filled quads, depth-tested, shaded by direction, fog; orbiting | scene: `f_aces! id list` (filled quads by id), a depth buffer |
| 4 | `voxels-world` | 4 by 2 by 4 chunks from value noise (2D heights, 3D caves optional), water, trees; each chunk's faces by id; X_eTaL culls chunks against the frustum | none beyond 3 |
| 5 | `voxels-walk` | walking: first-person camera, WASD and mouse look, gravity and jumping, collision against the blocks (swept box per axis) | scene: first-person camera; key down and up, mouse motion events (and grabbing the pointer) |
| 6 | `voxels-dig` | picking with the ray march; breaking and placing blocks; the edited chunk (and a neighbor at its border) remeshed and patched | none |
| 7 | `voxels-light` | sky light by a column scan, a torch's light spread in at most 15 rounds; faces carry a light level | scene: a light level per face |
| 8 | `voxels-game` | the mini game (below) | a small text overlay (score, time), if not drawn by X_eTaL |

Demos 1 and 2 need nothing new and could be written today. Each later
demo adds one native capability, tested in Rust with headless frames
like scene's today.

## The mini game

Small enough to finish, with a goal, a clock and an ending:
**Gem Hunt.** A seeded island world (64 by 32 by 64). Ten gems are
buried in stone below the surface; the hotbar has a pickaxe and three
block types. Dig to find them in three minutes; dirt is fast to
break, stone slow, water fills a hole it touches (one flow step a
tick, near the player only). A gem gives a point and a sound if the
audio extension is loaded. The score, the time and the gems left are
drawn on the frame. The end screen shows the score; the run's seed
and score can be kept in SQLite (a high-score table) with the sqlite
extension.

Everything in the game's rules is X_eTaL: the world, the generation,
which blocks break how fast, the water, the score. The window, the
drawing and the clock of frames are Rust.

## Rendering: CPU first, GPU only if needed

- **CPU (recommended):** a depth-buffered quad rasterizer in scene, at
  the window's size or a fixed internal size (for example 480 by 270,
  scaled up). A small world draws tens of thousands of quads; flat
  colors, no textures. It keeps what scene has now: no system
  libraries, headless frames, exact goldens, recordings with
  `XETAL_FRAMES`, the same code on macOS and Linux.
- **GPU (later, if the CPU falls short):** wgpu (pure Rust, Metal and
  Vulkan) behind the same `f_aces!`; X_eTaL would not change. SDL3,
  as in the APL game, would add a system library to install and buys
  nothing here: the hard parts the APL game needed SDL's C for (event
  unions, pipeline structs, font lists) are already Rust's side of the
  extension.

## What X_eTaL itself could improve

- **E1, a native hook:** without the text bridge, a chunk's faces
  would cross as a copy instead of formatting and parsing (0.35
  microseconds a number now).
- **Speed:** the exposed-face mask takes about 30 times Dyalog's time
  for the whole meshing; the demos fit in that, a large streamed world
  would not. Where the time goes (allocation per primitive, widening
  of Booleans) is not measured yet.

## Credits

The approach -- chunks as arrays, the six-rotation face mask, AABB
culling over all chunks at once, the ray march, the inverted table of
chunks -- is from Kyle Croarkin's "A Voxel Game" (MIT,
github.com/namgyaaal/avoxelgame) and his write-up "Notes on writing a
voxel game in Dyalog APL" (2026-03-06). Its textures (by Madeline
Vergani) are not used here.
