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
| `bp:s_till file`, `bp:v_ideo file` | a PNG of the frame; every frame as an MP4 (H.264) |
| `bp:e_xports name` | `.glb` (glTF), `.obj` (with `.mtl`) and `.blend` |
| `bp:report` | a report: each mesh's vertices and faces, the objects |

## Demos

1. `demos/hello.xtl` (`just demo blender hello`): the whole chain once.
   The program writes the launcher and a script that writes the names
   of the objects in Blender's starting scene to a report, runs the
   launcher, and reads the report back: `objects: Camera Cube Light`.
   Recorded as `videos/hello.webm`.

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (rust/tests) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
```

The Rust tests run a stand-in program in Blender's place; the goldens
`blender-demo-hello` and `blender-bpy-tiny` run Blender itself (in a
temporary directory): the second writes two cubes and a label as bpy
and checks what Blender made -- the mesh (16 vertices, 12 faces), the
objects, a still, `.glb`, `.obj`, `.mtl`, `.blend`. `blender-bpy-script`
pins the Python the library writes.
