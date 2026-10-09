# scene

Retained 3D scenes in a native window: an X_eTaL program puts objects
under ids it chooses (a polyline, segments, dots, filled quads) and
patches them later; Rust draws them in perspective from an orbit
camera, which the mouse turns -- quads shaded and depth-tested, lines
over them. The base of the visualizer and of the voxel demos.

- Package: `extensions/scene/` (`host = true`: linked into `xetal-x`,
  which serves its windows on the main thread)
- Facade: `lib/Scene.xtl`, recommended alias `sc:`
- Native crates: `winit` 0.30 (via the UI host) and `softbuffer` 0.4;
  drawn on the CPU (anti-aliased lines, quads behind a depth buffer),
  so a frame is the same with a
  window or without one. The idea -- a retained scene patched by stable
  ids, an orbit camera -- follows sw-ml-study/demo-extensions'
  native3d crates (which draw with wgpu).

## From X_eTaL

```
"sc:" u_se< "Scene"
w := "Cube" sc:o_pen! 480 480
k := w sc:c_amera! 0.6 0.45 5.0 0.8               # yaw pitch distance spin
x := ((f_loat w c_at 1) c_at 0.4 0.8 1.0) sc:s_egments! (r_avel e) s_elect c
e := sc:n_ext! w                                  # "frame", "key q", "close"
```

| Export | Type | What |
| ------ | ---- | ---- |
| `title sc:o_pen! w h` | `Num a => Char -> a -> Int` | a scene window `w` by `h` points; its id |
| `head sc:p_olyline! pts` | `(Num a, Num b) => a -> b -> Int` | put object: `head` is scene, object id, and optionally red green blue (0 to 1); `pts` an n by 3 matrix, joined in order; an id put again is replaced |
| `head sc:s_egments! pts` | as above | the points taken in pairs |
| `head sc:d_ots! pts` | as above | each point a dot |
| `head sc:q_uads! pts` | as above | the points in fours (4n by 3), each four the corners of a filled quad in order round it; shaded by how it faces a fixed light (tops brightest), nearer quads hiding farther ones; lines and dots draw over quads |
| `scene sc:e_ye! x y z yaw pitch` | as above | a first-person camera (on a dense screen drawn at the window's logical size and scaled up, so a Retina window costs no more than an ordinary one) at the eye, looking along yaw (0 toward -z, turning right toward +x) and pitch (up positive), radians; faces crossing the near plane are clipped, not dropped; dragging then turns nothing by itself -- it is reported by `sc:c_ontrols`; `sc:c_amera!` goes back to orbiting |
| `sc:c_ontrols scene` | `Num a => a -> Float` | how far the mouse was dragged since last asked (x, y pixels), then 1 or 0 for each key held: w a s d space shift and the arrows left right up down. The pointer is never grabbed. Headless, `XETAL_EVENTS` may hold `keydown w`, `keyup w` and `drag 10 0` |
| `scene sc:c_urve! radius` | as above | a curved horizon for the first-person camera: the world lowered by d^2 / 2R at a distance d along the ground from the eye, as on a small planet; 0 turns it off |
| `scene sc:s_ky! r g b` | as above | the background, which fog fades to; -1 -1 -1 is the dark default |
| `scene sc:f_og! near far` | as above | quads fade into the background between those depths (scene units from the camera); far at or below near turns fog off |
| `s sc:r_emove! o` | `(Num a, Num b) => a -> b -> Int` | remove object `o`: 1 if it was there |
| `s sc:c_amera! v` | `(Num a, Num b) => a -> b -> Int` | yaw, pitch (radians), distance, spin (radians a second) |
| `sc:n_ext! s` | `Num a => a -> Char` | the next event, waiting at most a frame: `frame`, `key NAME`, `close`; dragging orbits by itself |
| `sc:c_lose! s` | `Num a => a -> Int` | close the window |

## Without a screen

As canvas: `XETAL_HEADLESS=1` (events from `XETAL_EVENTS`, then
`close`) and `XETAL_FRAMES=DIR` (each frame saved as
`DIR/scene-ID-N.png`, at the size opened). The reg-rs test
`scene-demo-cube-headless` runs the cube that way and pins its frames.

## Recording

`videos/cube.webm` (and `.webp`): the cube turning, made headlessly
(`just videos scene`, `videos/cube.frames`).

## Demos

- `demos/cube.xtl` (`just demo scene cube`): a wireframe cube, its
  corners and edges X_eTaL arrays, turning; drag to turn it, q to quit.

### Voxels, piece by piece

The `voxels-*` demos build up to a small voxel game (the plan and its
measurements: `docs/voxels.md` in the repository), each one shown
before the next builds on it. Their shared X_eTaL is the library
`demos/Voxels.xtl` (`"vx:" u_se< "Voxels"`).

1. `demos/voxels-chunk.xtl` (`just demo scene voxels-chunk`, at the
   command line): a chunk is an array -- 16 by 16 by 16 block numbers
   built from a height field in a few array expressions: grass on
   top, dirt, stone, water below a level, sand at its shore, one tree
   on the hill. It counts the blocks, prints two slices as characters
   (`#` stone, `%` dirt, `"` grass, `:` sand, `~` water, `|` wood,
   `*` leaves), and finds what is seen from above -- the top of each
   column by a max-reduction down it -- drawing that and a slice as
   pictures. Recorded as `videos/voxels-chunk.webm`.
2. `demos/voxels-faces.xtl` (`just demo scene voxels-faces`): which
   faces to draw. A face shows where a solid block's neighbor is not
   solid: six rotations of the solid mask, each compared with the mask
   (the plane that wraps around made air), give every exposed face at
   once, and `vx:f_aces` lists them, one row each: x y z direction
   block (water adds its surface). Checked on shapes with known
   answers -- one block 1 a direction, a solid chunk 256, a hollow box
   452, a 3D checkerboard 2,048 -- then the chunk: 1,389 faces drawn
   instead of 10,446, as outlines colored by block, turning in the
   window. Recorded as `videos/voxels-faces.webm`.
3. `demos/voxels-solid.xtl` (`just demo scene voxels-solid`): the chunk
   drawn solid. The same faces, each made a quad in X_eTaL (its four
   corners from a 6 by 4 table by direction, `vx:q_uads`), one object
   of quads per kind of block; scene shades them by how they face the
   light, hides the farther behind the nearer, and fogs the distance.
   1,389 faces cross once as 16,668 numbers; after that only frames.
   Recorded as `videos/voxels-solid.webm`.
4. `demos/voxels-world.xtl` (`just demo scene voxels-world`; a new
   island each run, `just demo scene voxels-world --seed 1` for the
   recorded one): a world of chunks. An island 64 by 32 by 64 -- 32
   chunks -- from value noise made by linear algebra (random heights
   on grids every 16, 8 and 4 blocks, each interpolated as M G M'),
   the chunk's rules applied to the whole world at once, and trees
   planted by folding one tree over a list of places. The faces are
   found over the whole array, so faces between chunks hide each other
   (18,568 drawn of 403,092), and sent chunk by chunk under ids (10 *
   chunk + block, 123 objects) -- the ids a later demo patches when a
   block changes. About 0.4 s to build and mesh; scene draws it in
   about 13 ms a frame. Recorded as `videos/voxels-world.webm`.
5. `demos/voxels-walk.xtl` (`just demo scene voxels-walk`): walking on
   the island in the first person -- W A S D, Space to jump, Shift to
   run, drag the mouse or the arrows to look, c for the chunks in
   view, H back to the start, q to quit. The player is nine numbers; each frame X_eTaL reads
   `sc:c_ontrols`, turns, falls, jumps, and moves one axis at a time
   against the solid blocks the player's box would overlap, stopping
   at the face it hits, then sets `sc:e_ye!`. The chunks in view: each
   chunk's box against the planes of the view, the planes turned into
   the world by an inner product. Water is not solid yet (swimming
   comes later). Recorded from a scripted walk as
   `videos/voxels-walk.webm`.
6. `demos/voxels-endless.xtl` (`just demo scene voxels-endless`): an
   endless world -- no edges, no walls. The keys are voxels-walk's:
   W A S D, Shift to run, Space to jump or swim up, drag or the arrows
   to look, H back to the start, q to quit. The land comes from value noise
   hashed from the coordinates (`vx:h_eightsAt`), so any column of 16
   by 16 blocks is made by itself (`vx:p_atch`, about 35 ms, meshed with
   a border of its neighbors so the seams hide) and fits its
   neighbors. Columns are made around the player, the nearest missing
   one each frame, and dropped beyond reach; collision asks the columns
   themselves; fog hides the edge of what is made, under a blue sky and
   a curved horizon. scene skips objects behind the eye, beside the
   view or past the fog. The world and the player are the library
   `demos/Endless.xtl` (`"en:" u_se< "Endless"`), shared with
   voxels-fly: each column is made over three frames -- its blocks and
   solid mask, its faces, sending them -- and at most one far column is
   dropped a frame, so no frame waits long (a flight: median 1 ms of
   X_eTaL a frame, 95% within 14 ms; making a column in one frame had
   taken 40 to 68). Recorded from a scripted run as
   `videos/voxels-endless.webm`.
7. `demos/voxels-fly.xtl` (`just demo scene voxels-fly`): flying over the
   endless world. F switches between walking and flying; flying moves
   level whatever you look at (W A S D by the heading, Space up, Shift
   down, 15 blocks a second, no gravity, the blocks still stopping you),
   so you can look down at the land as you cross it; F again drops you
   back to walking. Recorded from a scripted flight as
   `videos/voxels-fly.webm`.
8. `demos/voxels-rubik.xtl` (`just demo scene voxels-rubik`): a
   Rubik's cube of voxels -- 26 dark cubies with their stickers. The
   cube is arrays (the library `demos/Rubik.xtl`, `rb:`): its 54
   stickers, each a cubie's position and the way it faces, and each of
   the twelve quarter turns a permutation of them computed from the
   geometry (the layer's stickers rotated, matched back, and `g_rade`
   turning "where each goes" into "where each comes from"). Checked:
   each turn a permutation, four of one or a turn and its undoing
   solved, R U R' U' of order 6, R U of order 105, U and R turning the
   right way. Keys: u d r l f b turn a face clockwise, with Shift
   counterclockwise, z undoes (all the way back to solved), Space
   scrambles (25 random turns, never the same face twice in a row, so
   always a real position), 0 resets. Recorded as
   `videos/voxels-rubik.webm`.
9. `demos/voxels-rubik-turn.xtl` (`just demo scene voxels-rubik-turn`):
   a quarter turn, seen. One layer -- a "plane" of 9 cubies -- turns a
   quarter revolution while the rest of the cube stays still: its
   cubies' bodies and stickers rotated a little more each frame by a
   rotation matrix (`rb:r_otation`, one inner product, `rb:d_raw`),
   then the colors permuted once it is done. The top, right and front
   layers in turn, each a quarter turn one way and back (U U' R R' F
   F'). Recorded as `videos/voxels-rubik-turn.webm`.

## Build and test

```sh
just build      # xetal-x with scene linked in
just test       # Rust tests (projection, ids, camera, quads, depth, fog) and reg-rs tests (headless)
```
