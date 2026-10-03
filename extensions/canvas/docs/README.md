# canvas

A native window showing X_eTaL arrays as pixels, with keys, clicks and
frame ticks back -- the first extension served by the UI host
(`crates/xetal-ext-ui`), and the base of the media saga's visualizer.

- Package: `extensions/canvas/` (`extension.toml` says `host = true`:
  canvas is linked into `xetal-x`, not loaded as a shared library)
- Facade: `lib/Canvas.xtl`, recommended alias `cv:`
- Native crates: `winit` 0.30 (the window and its events, via the UI
  host) and `softbuffer` 0.4 (a CPU pixel buffer in the window)

## From X_eTaL

```
"cv:" u_se< "Canvas"
w := "Life" cv:o_pen! 512 512        # a window; its id
s := w cv:s_how! board               # draw an array
e := cv:n_ext! w                     # "frame", "key q", "click 3 7", "close"
c := cv:c_lose! w
```

| Export | Type | What |
| ------ | ---- | ---- |
| `title cv:o_pen! w h` | `Num a => Char -> a -> Int` | a window `w` by `h` points; its id |
| `id cv:s_how! p` | `(Num a, Num b) => a -> b -> Int` | draw an n by m matrix (grey) or an n by m by 3 array (red, green, blue), each 0 (dark) to 1 (bright); a Bool matrix draws its 1s white; scaled to the window, keeping its shape |
| `cv:n_ext! id` | `Num a => a -> Char` | the next event, waiting at most a frame (1/60 s): `frame`, `key NAME` (`key Space`, `key q`, `key ArrowLeft`), `click ROW COL` (in the array shown, from 1), `close` |
| `cv:c_lose! id` | `Num a => a -> Int` | close the window |

A program loops with `p_ower` over a bounded number of frames; a frame
after quitting does nothing (`demos/life.xtl`).

## How it works

On macOS a window must live on the main thread, but X_eTaL runs a
program on a worker thread. `xetal-x` therefore runs the program on a
thread of its own and gives its main thread to the UI host, which
starts a winit event loop the first time a program opens a window
(other programs run exactly as before: the gate checks every vendored
demo). `cv:o_pen!` and `cv:s_how!` are jobs run on the main thread;
the window's events go to a queue that `cv:n_ext!` reads from the
program's thread. No callbacks: the program pulls.

## Demos

- `demos/life.xtl` (`just demo life`): Conway's Life, 96 by 96, in a
  window; Space for a new board, q or closing the window to quit.

## Build and test

```sh
just build      # xetal-x with canvas linked in
just test       # Rust tests (pixels, scaling) and reg-rs tests (headless)
just smoke      # opens a window briefly: needs a desktop session
```

The gate's tests never open a window: they check the conversion of
arrays to pixels and the scaling, the facade's types, and errors.
`just smoke` and the demo do.
