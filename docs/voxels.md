# Voxels: a mini game in X_eTaL, built up demo by demo

An analysis of `../avoxelgame-fork` (Kyle Croarkin's voxel game in
Dyalog APL, MIT) and of how this repository could show voxel graphics
from X_eTaL: what carries over, what our native windows can and
cannot do today, measured costs, the chunk size to use, and a series
of `voxels-*` demos that build up to a small game, each one runnable,
tested and recorded before the next. Written 2026-10-07; nine demos
of the world are built -- voxels-chunk, voxels-faces, voxels-solid,
voxels-world, voxels-walk, voxels-endless, voxels-fly, voxels-dig and
voxels-water (see "Built so far") -- and light and the game follow
(see "The game" for the order).

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

| # | Demo | Status | Shows | New native work |
| - | ---- | ------ | ----- | --------------- |
| 1 | `voxels-chunk` | built | a 16-cube as a 3D array: a height field made into layers (stone, dirt, grass), water below a level; counts per block type; slices drawn with `[]G_RID` | none |
| 2 | `voxels-faces` | built | the exposed-face mask by six rotations (the Life idiom), padded with neighbors; the face list as an n by 5 array; face counts for solid, hollow and checkerboard chunks; the faces drawn as wireframe squares in scene, orbiting | none (scene segments) |
| 3 | `voxels-solid` | built | the same chunk drawn solid: filled quads, depth-tested, shaded by direction, fog; orbiting | scene: filled quads by id (`sc:q_uads!`), a depth buffer, fog (`sc:f_og!`) |
| 4 | `voxels-world` | built | 4 by 2 by 4 chunks from value noise (2D heights), water, trees; each chunk's faces by id | none beyond 3 |
| 5 | `voxels-walk` | built | walking: first-person camera, WASD and mouse look, gravity and jumping, collision against the blocks (swept box per axis); X_eTaL culls chunks against the frustum | scene: first-person camera; key down and up, dragging to look (the pointer never grabbed) |
| 6 | `voxels-dig` | built | picking with the ray march; breaking and placing blocks; the edited chunk (and a neighbor at its border) remeshed and patched | scene: an overlay of flat rectangles (crosshair, hotbar), click events |
| 7 | `voxels-light` | planned | sky light by a column scan, a torch's light spread in at most 15 rounds; faces carry a light level | scene: a light level per face |
| 8 | `voxels-game` | planned | the mini game (below) | a small text overlay (score, time), if not drawn by X_eTaL |

(The order changed as the work went: after voxels-world came
voxels-walk, then voxels-endless and voxels-fly, added at the user's
request; see "The game" for the order from there.) Demos 1 and 2
needed nothing new. Each later
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
  Vulkan) behind the same `sc:q_uads!`; X_eTaL would not change. SDL3,
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

## Built so far

Demos 1 to 4 are in `extensions/scene/demos/` with the shared library
`Voxels.xtl`, each with a golden (headless frames pinned where it
draws) and a recording (the scene page lists them):

| Demo | What it showed |
| ---- | -------------- |
| `voxels-chunk` | a chunk from a height field in a few elementwise expressions; the view from above by a max-reduction down each column |
| `voxels-faces` | the six-rotation mask checked on known shapes (1, 256, 452, 2,048 faces a direction) and the chunk's 1,389 faces of 10,446 |
| `voxels-solid` | the chunk as shaded, depth-tested quads: 16,668 numbers crossed once |
| `voxels-world` | 32 chunks, 64 by 32 by 64, from value noise made as M G M'; faces over the whole world (18,568 of 403,092) sent per chunk under ids; 0.4 s to build and mesh, about 13 ms a frame to draw |

What changed from the plan above:

- scene gained generic filled quads (`sc:q_uads!`, shaded by a fixed
  light, behind a depth buffer) and fog (`sc:f_og!`) rather than a
  voxel-specific `f_aces!`: X_eTaL turns faces into quads.
- The world's faces are found over the whole world array at once
  (55 ms of the 0.4 s), which also settles the chunk-border question
  for a fixed world; remeshing one chunk after an edit (`voxels-dig`)
  will need its neighbors' border planes.
- Frustum culling moved to `voxels-walk`: an orbiting camera always
  sees the whole island.
- The CPU renderer is fast enough: about 13 ms for 18,568 quads at
  640 by 560, so no GPU is needed for the game.

Demos 5 to 7 followed (the scene page lists them all with their keys):

| Demo | What it showed |
| ---- | -------------- |
| `voxels-walk` | the first person: the player as nine numbers, gravity, jumping and a swept box per axis in X_eTaL; a first-person camera, held keys and dragging in scene (the pointer never grabbed), faces clipped at the near plane; the chunks' frustum test as inner products; swimming (Space rises, a jump climbs out) and H for home |
| `voxels-endless` | no edges: value noise hashed from the coordinates, so any 16 by 16 column is made alone and fits its neighbors; columns made around the player and dropped behind; a sky, fog and a curved horizon in scene; culling by bounding box |
| `voxels-fly` | F to fly: level by the heading, Space up, Shift down, the blocks still stopping you; on a Retina screen scene draws at the logical size and scales up; trees placed by their own cells only (three times faster, and their crowns rounded as meant) |
| `voxels-rubik` | a Rubik's cube (the user): 54 stickers, each quarter turn a permutation computed from the geometry and checked (order 4, inverses, R U R' U' of order 6, R U of order 105, directions); keys turn faces, z undoes to solved, Space a valid scramble |
| (smoother streaming) | the user found the edges laggy: a column took 40 to 68 ms in one frame; now it is made over three frames (blocks, faces, sending), at most one far column is dropped a frame, the solid masks are separate boxes (nothing big is copied), and faces are found for the column's own cells with cached coordinates -- a flight's frames: median 1 ms of X_eTaL, 95% within 14 ms; the world and the player became the library `Endless.xtl` |
| `voxels-dig` | digging and building: the pick as 100 points along the view ray looked up in the masks of the 1 to 4 columns they cross; an edit is a row (x y z block) the world keeps, newest first, and every column is made with the edits in it and its border (one `i_ndexOf` over its 10,368 cells), so an edited column is simply made again -- its mask at once, its faces and objects over the next two frames -- with a neighbor when the cell is on their border, and edits outlast a column dropped and made again; scene gained an overlay (the crosshair and the hotbar) and clicks |
| `voxels-water` | water that flows: a cellular automaton on the frontier of cells it has just reached -- fall into air below (level 7), else spread one level lower into air at the sides -- with the sea and lakes as sources; reached cells are edits, each column keeps a water mask so a frontier is classified at once; ticks paced (every 6 frames at most, after the changed columns are drawn); a lake on a stepped hill's terrace, built as edits, runs down the steps when its rim is dug (the user's showcase); scene draws translucent quads (alpha), water's sides and bottom against air |


## The game: walking, flying, building, digging, light, water

Designed 2026-10-07 after demos 1 to 4 (the user: "how could we have a
voxel game that allows walking, fly-through, building, digging,
lighting, water flow?"). The numbers it rests on, measured above: about
0.4 microseconds a cell for whole-array work in X_eTaL (1.6 ms a
16-cube), 0.35 microseconds a number across the bridge, about 13 ms a
frame for scene to draw 18,568 quads. The program and the window run
on different threads, so drawing does not take X_eTaL's time; a frame
is 16 ms.

**The loop.** Each frame the program pulls one event, applies the
input, moves the player, sets the camera, and then does at most one
bounded job from a queue -- generate a chunk, remesh a chunk, a few
rounds of light, a water tick -- so no frame does more than about 8 ms
of X_eTaL.

**The world** is a list of chunk arrays (16-cubes, boxed, keyed by
chunk), not one big array: X_eTaL has no indexed assignment (values
are immutable, lang-choices M1), so an edit rebuilds one 4,096-cell
chunk (about 2 ms), never the world.

| Feature | In X_eTaL | In scene (Rust) |
| ------- | --------- | --------------- |
| walking | the player as a few numbers (position, velocity, yaw, pitch); gravity and jumping; collision as a swept box per axis against the dozen cells around the player, read from the solid mask | a first-person camera (eye position, yaw, pitch); held keys, dragging to look (the pointer never grabbed); quads clipped at the near plane (a face beside the eye must not vanish) |
| fly-through | the same with gravity and collision off; chunks generated ahead of the player from the queue, one a frame (about 10 ms each: terrain, faces, sending) | objects outside the view skipped by their bounding boxes; fog hides the edge; an internal resolution below the window's if the frame time needs it (wgpu behind the same calls only if the CPU falls short) |
| digging and building | the pick: 100 points marched along the view ray, rounded to cells, the first solid one -- one expression; the edit rebuilds one chunk, its faces remeshed with the neighbors' border planes and its objects replaced by id | a crosshair; the hotbar |
| lighting | sky light: a running scan down each column; block light (torches) and soft sky light: rounds of "the brightest neighbor (six rotations) minus 1", blocked by solid blocks, about 2 ms a round a chunk, 15 rounds, only after a change, spread over frames; a face takes the light of the air cell in front of it | a brightness per quad (a fourth column) |
| water flow | a cellular automaton like Life on levels 1 to 7: water falls into air below, else spreads sideways one level lower, sources stay full; a few rotations and comparisons a tick, about 15 ms a chunk, every few frames, only on chunks where water changed | water drawn after the solid blocks, translucent (alpha) -- opaque blue first |

The two walls the APL game hit -- light and water, "spread from a point
until blocked" -- are fixpoints over the whole array there; here they
run over 16-cubes, only where something changed, a few rounds a frame.

**An ask, not a blocker:** an "amend" that returns a new array with
some indexes changed (APL's `@`) would make an edit cost the cells
changed instead of the chunk; it fits immutable values.

**The order**, each demo shown before the next:

| # | Demo | Status | Adds |
| - | ---- | ------ | ---- |
| 1 | `voxels-walk` | built | first-person camera, held keys, mouse look, near-plane clipping in scene; gravity, jumping, collision in X_eTaL; X_eTaL's frustum test of the chunks |
| 2 | `voxels-endless` | built | (added) walking with no edges: terrain from noise hashed from the coordinates, so any chunk can be made alone and fits its neighbors; chunks generated around the player and dropped behind; fog at the loaded radius and a curved horizon (distant land lowered by d^2/2R, so the world looks like a small planet); culling by bounding box in scene |
| 3 | `voxels-fly` | built | flying over the endless world: F to fly, level by the heading, Space up, Shift down |
| 4 | `voxels-dig` | built | the ray-march pick, breaking and placing, per-chunk remeshing and patching; crosshair and hotbar |
| 5 | `voxels-water` | built | water flowing by a cellular automaton into what is dug; a hill with a lake on a terrace, held above the sea -- dig the ground downhill of it and the water runs out and falls down the slope (the user); translucent water |
| 6 | `voxels-light` | planned | sky light by a column scan, torches spreading in rounds; brightness per quad |
| 7 | `voxels-game` | planned | Gem Hunt, with all of the above |

## Credits

The approach -- chunks as arrays, the six-rotation face mask, AABB
culling over all chunks at once, the ray march, the inverted table of
chunks -- is from Kyle Croarkin's "A Voxel Game" (MIT,
github.com/namgyaaal/avoxelgame) and his write-up "Notes on writing a
voxel game in Dyalog APL" (2026-03-06). Its textures (by Madeline
Vergani) are not used here.
