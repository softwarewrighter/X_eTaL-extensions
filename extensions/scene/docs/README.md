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

## Build and test

```sh
just build      # xetal-x with scene linked in
just test       # Rust tests (projection, ids, camera) and reg-rs tests (headless)
```
