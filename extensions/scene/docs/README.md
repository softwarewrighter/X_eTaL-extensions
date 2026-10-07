# scene

Retained 3D line and point scenes in a native window: an X_eTaL program
puts objects under ids it chooses (a polyline, segments, dots) and
patches them later; Rust draws them in perspective from an orbit
camera, which the mouse turns. The base of the visualizer.

- Package: `extensions/scene/` (`host = true`: linked into `xetal-x`,
  which serves its windows on the main thread)
- Facade: `lib/Scene.xtl`, recommended alias `sc:`
- Native crates: `winit` 0.30 (via the UI host) and `softbuffer` 0.4;
  drawn on the CPU (anti-aliased lines), so a frame is the same with a
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

## Build and test

```sh
just build      # xetal-x with scene linked in
just test       # Rust tests (projection, ids, camera) and reg-rs tests (headless)
```
