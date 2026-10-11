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

The Rust tests run a stand-in program in Blender's place; the golden
`blender-demo-hello` runs Blender itself (in a temporary directory).
