# three

Interactive 3D in a browser. An X_eTaL program writes a scene as the
calls of a player -- the same text the blender extension's Cycles.xtl
writes for a ray-traced film -- and here a player written with
Three.js draws those calls in real time: ride the camera's flight, fly,
or walk. The first scene is the neon city: 240 lots worked out as
arrays from a seed, so another seed is another city, built in the page
by X_eTaL compiled to WebAssembly.

Live: <https://softwarewrighter.github.io/X_eTaL-extensions/three/live/>

- Package: `extensions/three/` (`extension.toml`; library
  `xetal_ext_three`: `libxetal_ext_three.dylib` on macOS, `.so` on Linux)
- Facade: `lib/Three.xtl`, recommended alias `th:`
- Native crates: none beyond the SDK (in a browser, `js-sys` and
  `wasm-bindgen` to reach the page)
- Vendored: Three.js 0.170.0 (MIT), the passes the player uses and
  its glTF loader, in `web/static/vendor/three/` with its license;
  nothing is loaded from another site

## Functions

| Native name | Arity | X_eTaL type | What |
| ----------- | ----- | ----------- | ---- |
| `about` | 0 | `Unit -> Char` | the extension's name and version |
| `show` | 1 | `Char -> Int` | draw a scene in the page, in place of the one before; how many calls it holds. From the command line there is no page and nothing is drawn |
| `page` | 2 | `Char Char -> Int` | write a directory to serve -- the page, the players, Three.js and the scene's calls; how many files. Under the working directory (or `XETAL_THREE_ROOT`) |

## From X_eTaL

```
"th:" u_se< "Three"
"nc:" u_se< "NeonCity"
scene := (7, "media", 1280, 720, 30, 24, "GPU") nc:c_ity 0   # seed 7: the film's city
th:s_how! scene                                    # in a page: draw it
"work/city" th:p_age! scene                        # from the command line: a directory to serve
```

Run with `xetal-x --ext extensions run PROGRAM` (see the repository's
docs/bridge.md); NeonCity.xtl is in `extensions/blender/demos`.

## One scene, two players

A scene is text: one call a line, each a table of numbers.

```
facade('skin 0', nums('0.04 0.04 0.06', 3)[0], ...)
boxes('towers 0', nums('-19.5 -234.5 30.7 14.5 16.5 61.5 0.0 ...', 7), 'skin 0')
fleet('cars', 'car', nums('-2.6 232.5 0.0 -2.6 76.0 0.0 ...', 6))
fly(nums('0.0 3.0 6.0 9.0 12.0 15.0', 6)[0], ...)
```

The blender extension puts its player, `cy:head`, in front of that
text and Blender runs it as Python: Cycles ray-traces a film. The same
text is JavaScript, and `web/static/player.js` defines the same
functions with Three.js: `stage`, `look`, `sky`, `bloom`, `surface`,
`neon`, `facade`, `wet`, `screen`, `boxes`, `sign`, `model`, `fleet`,
`lamps`, `fog`, `steam`, `fly`. Each reads its numbers and decides
nothing about the scene.

| In the film (Cycles) | Here (Three.js) |
| -------------------- | --------------- |
| puddles reflect by tracing rays | the street mirrors the city by drawing it a second time, upside down |
| screens, signs and lamps light walls and street, with shadows | lamps light the street and plain surfaces; nothing casts a shadow; screens and neon glow (bloom) but do not light |
| haze scatters light; steam is a volume | haze is a fade with distance; steam is a flat sheet facing you |
| windows and puddles from Blender's noise | the same rules with other noise, so they fall in other places |
| a video on a screen (`cy:r_eel`), a slide show as a video (`cy:s_lides`) | the video (one decoder, however many screens show it); the slides as stills faded one into the next, no decoder |
| depth of field, motion blur | neither |

## A game's board

A second player, `web/static/board.js`, draws a board: a scene whose
first call is `view`. It was asked for by a game written in X_eTaL
(sw-fun/evolving-game), whose view library writes these four calls,
a line each, with x east, z south and y up, in tiles:

```
view(848, 527, '#1d1f2b')
camera(nums('9.525 0.0 9.525', 3)[0], nums('0.612 0.5 0.612', 3)[0], 28.284)
tiles('road', '#b9a37a', nums('10.5 4.5 0.5 10.5 5.5 0.5 ...', 3))
piece('walker', nums('6.5 10.0 0.5 0.0 -1.0', 5)[0], 'walk', 10.6, nums('6.5 9.875 0.5', 3)[0], 0.05)
```

| Call | What it draws |
| ---- | ------------- |
| `view(w, h, background)` | the picture's size in its own units, and the color behind the board; the whole view is fitted into the page's element |
| `camera(target, direction, scale)` | no perspective: looking at `target` from along `direction`, `scale` view units to a tile. With the direction 30 degrees up along the board's diagonal, a board point (x, z, y) is drawn where an isometric picture's map puts it, so a click goes back to a tile through that map's inverse |
| `tiles(kind, color, nums(.., 3))` | every tile of a kind in one instanced call: `x z y` is a tile's middle and its top, and the tile is a column from the ground to that top, its south and east faces darker |
| `piece(model, from, clip, t, to, dt)` | a piece: `from` is `x z y hx hz` (where it is, the way it faces), `clip` and `t` the model's animation and the time into it, `to` is `x z y` where it will be in `dt` seconds |

- A piece's model is `models/NAME.glb` beside the page (`models` in
  the page's setup names another folder). Until it has come, and when
  there is none, the piece is a plain shape: a body, a head, and a
  nose the way it faces.
- Between a program's ticks the player moves each piece from `from` to
  `to` over `dt` seconds and runs its clip on from `t`, at the
  display's rate: the program runs at its own tick, not once a frame.
- A scene shown again changes the board in place: a `tiles` line that
  is the same text is kept as it is, and pieces are matched by model
  and order (the second `walker` is the second `walker` of the scene
  before), so a tick costs only what moved.
- A click is given to the page's `onPoint(x, y, event)` in the view's
  units. The player decides nothing about the game: which tile that is
  is the program's to work out.
- `?t=SECONDS` holds every piece that long after its tick (a still).

`th:s_how!` and `th:p_age!` take a board as they take a city: the
page's controls (`live.js`) choose the player by the scene's first
call. `just ext three board` writes `tests/board.xtl`'s small board to
`work/board`; `just ext three serve board` serves it.

## The pictures on the screens

About a third of the city's screens play pictures worked out from the
time (bars, rings, rain, tiles). The rest show real ones, small copies
kept in `web/static/media` (37 files, about 2 MB; nothing comes from
another site):

- two slide shows, each still held four seconds and faded into the
  next: illustrations from the author's blog, and X_eTaL demos;
- six short looping videos: a matrix product flown through (the great
  screen), a voxel world, a Rubik's cube turning, the CoSy logo, Life
  on a torus, a stone turning -- this repository's recordings and two
  of the blog's animated pictures.

`media.toml` lists the sources and `just ext three media`
(`scripts/city-media.py`) makes the copies; `SOURCES.md` beside them
says where each came from. NeonCity.xtl says which kind of screen
plays what (`nc:s_creens`). A slide show costs a browser nothing to
decode, and a video is decoded once however many screens show it.

## The page

- **Ride** follows the film's flight, again and again. **Fly** and
  **Walk** start where you are: click the picture, then W A S D and
  the mouse; E and Q rise and sink (flying); Shift is faster; Esc
  frees the mouse. Towers, bridges and pylons stop you, flying or
  walking.
- **Seed**: Build makes the city of that number, Another a random one.
  X_eTaL runs in the page and takes a fraction of a second.
- **About** opens what a footer would say -- copyright, license, the
  repository, the X_eTaL commit, and the build's host, commit and time -- with
  the program that ran for the seed and, in sections that open, the
  sources of NeonCity.xtl and Three.xtl, all drawn as X_eTaL draws
  them, in color.
- `?t=SECONDS` holds the flight at a moment, `?hud=0` hides the
  controls, `?seed=N` builds that city first, `?mode=fly` or `walk`
  starts there, `?about` opens About, `?bench` puts the frame rate and
  the slowest frame in the page's title (to measure from outside).
- A video plays only while one of its screens is in view and near,
  and holds its frame otherwise; slide pictures go to the graphics
  card as they arrive; a machine that falls under 40 frames a second
  is given fewer pixels to draw.

## Demos

1. `demos/city.xtl` (`just ext three city`): the film's city written
   as a directory to serve, `work/city`; `just ext three serve city`
   serves it at <http://127.0.0.1:8470/>. No WebAssembly: the calls
   were worked out on the command line.
2. The live page (`web/`, `just ext three live-serve`): X_eTaL and
   this extension compiled to WebAssembly, the city built from the
   seed you give. `just ext three publish` copies it to
   `pages/three/live` for the site.

`just ext three ride` records the ride from the page itself (headless
Chrome, no window) as `work/city/ride.mp4`.

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (rust/tests) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
just live       # the live page, with trunk, into web/dist
```

The Rust tests check the descriptor, that each player defines every
call its vocabulary writes, and that the vendored Three.js is whole.
The goldens: `three-list`; `three-tiny` (three calls counted, a page
of 19 files written); `three-board` (a board of six calls counted and
written as a page); `three-outside` (a path outside the working
directory is refused); `three-demo-city` (the city: 84 calls). The
page's crate (`web/`) runs its program natively for two seeds and
checks the decorated source. What the player draws is checked by eye,
from headless Chrome's pictures.
