<p align="center">
  <img src="images/modern-xetal-logo.jpg" alt="X_eTaL: X underlined, a raised e, T, a raised a, L" width="360">
</p>

# X_eTaL extensions

<p align="center">
  <b><a href="https://softwarewrighter.github.io/X_eTaL-extensions/">The demos, recorded</a></b>
  -- X_eTaL programs calling native Rust extensions, and how to run them yourself
</p>

<p align="center">
  <img src="extensions/sqlite/videos/notebook.webp" alt="The data notebook running at the command line" width="760">
</p>

Native extensions for [X_eTaL](https://github.com/softwarewrighter/X_eTaL),
the eXperimental Extensible Typed Array Language: small Rust libraries
that give X_eTaL programs what the interpreter cannot do by itself --
a clock, hashing, regular expressions, fast linear algebra, image
files -- each used from X_eTaL like any other library.

## Start here

```sh
git clone https://github.com/softwarewrighter/X_eTaL-extensions
cd X_eTaL-extensions
cargo build --workspace          # Rust is all it needs: X_eTaL is vendored
just demos                      # every demo: EXT NAME and what it shows
just demo hello tour            # X_eTaL arrays handed to Rust and back
just demo sqlite notebook       # CO2 at Mauna Loa: SQLite + X_eTaL, pictures in work/draw/
just demo canvas life           # Life in a native window (Space: new board, q: quit)
just demo audio spectrum        # the music visualizer: X_eTaL analyses, Rust plays and draws
just walkthrough                # a fresh clone, built and run, checked against the goldens
```

([`just`](https://github.com/casey/just) runs the recipes.) What works
today, extension by extension: [docs/status.md](docs/status.md).

## What this is

**Libraries extend the vocabulary; macros extend the language; native
extensions extend the machine.** X_eTaL is extensible three ways:
ordinary `.xtl` libraries add typed array code, `.xtlm` macro
libraries add notation that expands into ordinary X_eTaL (both in
[X_eTaL-libraries](https://github.com/softwarewrighter/X_eTaL-libraries)),
and native extensions -- this repository -- add capabilities the
language should not reinvent: a database engine, a clock, image
codecs, fast numerical code. Each is a Rust library behind an
ordinary, statically typed X_eTaL facade, so a program imports it like
any other library and the type checker sees its functions' types; the
language core stays small.

For example, the data notebook loads a CSV into SQLite, lets SQL do
what it is good at (selecting, grouping) and X_eTaL what it is good at
(whole-array arithmetic: the yearly rise of CO2 at Mauna Loa, a
least-squares line, its residuals, a histogram), and draws the result:

```
"sq:" u_se< "Sqlite"
db sq:i_mport "co2=demos/data/co2-mlo-annual.csv"
m := db sq:n_ums "select year, mean from co2 order by year"
c := 2 s_elect_2 m
d := (1 d_rop c) - -1 d_rop c            # the rise each year, as one array
```

A smaller example:

```
"hx:" u_se< "Hello"
hx:s_hout "x_etal"                          # X_ETAL, upper-cased in Rust
hx:s_um (r_ange 10) '* t_able r_ange 10      # 3025.0, a 10 by 10 array summed in Rust
```

The program does not know whether `hx:s_um` is X_eTaL, Rust or C.
Behind the facade:

- each extension is a Rust shared library (`.dylib` / `.so`) that
  exports one C-ABI descriptor (ABI V1): its name, version and a
  table of typed functions;
- a loader validates the descriptor, copies it, and calls the
  functions, containing errors and panics;
- a facade library (`Sqlite.xtl`) gives each function an X_eTaL name
  and type.

X_eTaL does not yet have a native hook, so for now programs that use
extensions run with the bridge host `xetal-x` instead of `xetal` (see
`docs/xetal-asks.md`, E1). `xetal-x` is the vendored `xetal` -- every
subcommand and message the same -- plus extensions:

```sh
just ext-list                 # the extensions here and their functions
just run-x prog.xtl           # xetal-x --ext extensions run prog.xtl
```

How the channel works, and what its errors look like:
[docs/bridge.md](docs/bridge.md). When X_eTaL gains the hook, the facades
change and the programs do not.

## Extensions

**Status: experimental.** Release 1 is **hello, clock and sqlite**,
with the data notebook as its flagship demo. The native extension ABI
and the `ext:` bridge (`xetal-x`) are previews: X_eTaL will call native
code itself one day (docs/xetal-asks.md, E1), and programs will not
change when it does. Canvas (a native window) is the first piece of
the media work -- audio and 3D, like the MP3 visualizer -- which
follows the ecosystem's launch, as do the other roadmap extensions.

| Demo | Extensions | What it shows | Status |
| ---- | ---------- | ------------- | ------ |
| [data notebook](extensions/sqlite/docs/README.md#the-data-notebook) | sqlite | CO2 at Mauna Loa: a CSV in SQLite; SQL groups, X_eTaL computes the yearly rise, a least-squares line and its residuals, a histogram; SVG pictures | release 1, done |
| X_eTaL on the web | web, sqlite | a live page recomputing Life or Mandelbrot per request; a TodoMVC stored in SQLite | roadmap |
| photo lab | image, linalg | a photo as an array: filters, edges, SVD compression; PNGs out | roadmap |
| fetch and analyze | http | live earthquake data summarized and plotted | roadmap |

| Extension | Facade | What | Crates | Status |
| --------- | ------ | ---- | ------ | ------ |
| [hello](extensions/hello/docs/README.md) | `Hello` | the smallest proof of the boundary | -- | done: facade, tests, demo |
| [clock](extensions/clock/docs/README.md) | `Clock` | wall-clock and monotonic time; the bridge's cost | std | done: facade, tests, cost demo |
| [audio](extensions/audio/docs/README.md) | `Audio` | decode Ogg Vorbis, MP3, WAV; play; read what plays as arrays | symphonia, cpal | done: the music visualizer (`just demo audio spectrum`) |
| [canvas](extensions/canvas/docs/README.md) | `Canvas` | a native window showing arrays as pixels; keys and clicks back | winit, softbuffer | done: Life demo (media work continues after the launch) |
| [scene](extensions/scene/docs/README.md) | `Scene` | retained 3D lines and points in a native window, patched by id; orbit camera | winit, softbuffer | done: cube demo (the visualizer's base) |
| [sqlite](extensions/sqlite/docs/README.md) | `Sqlite` | execute and query SQLite files; CSV import | rusqlite | done: facade, CSV import, tests, the data notebook |
| web | `Web` | serve HTTP: the program takes each request and replies | axum, tokio | roadmap |
| image | `Image` | images to and from arrays | image | roadmap |
| linalg | `Linalg` | solve, inverse, least squares, eigenvalues, SVD | nalgebra | roadmap |
| http | `Http` | bounded GET | ureq | roadmap |
| digest | `Digest` | SHA-256, CRC-32 | sha2, crc32fast | roadmap |

Each extension is a self-contained directory, `extensions/NAME/`: its
manifest (`extension.toml`), its own `justfile`, its Rust crate
(`rust/`), its X_eTaL sources (`lib/`), its reg-rs tests (`tests/`),
its docs (`docs/`) and its demos (`demos/`). `just new-ext NAME ALIAS
"what"` starts one from `templates/extension/`; `just ext NAME RECIPE`
runs one of its recipes from the repository root.

## Writing an extension

An extension is plain Rust: functions taking the X_eTaL arguments and
returning a value or an error, listed once with their arity, X_eTaL
type and a line of documentation. The SDK generates the C boundary.

```rust
use xetal_ext_sdk::{OwnedError, Value, text};

fn shout(args: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(text(&args[0])?.to_uppercase()))
}

xetal_ext_sdk::xetal_extension! {
    name: "hello",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        shout: 1, "Char -> Char", "The text in upper case.";
    }
}
```

See [`extensions/hello`](extensions/hello/rust/src/lib.rs) and the
[ABI](docs/abi-v1.md).

## Loading an extension from Rust

A host loads extensions with `xetal-ext-loader`: a package directory
(its `extension.toml`) names the library; the registry finds it
(`native/<platform>/` in the package, then any build directories
given), validates its descriptor against ABI V1 and the manifest, and
calls its functions by name with owned values:

```rust
use xetal_ext_loader::{Package, Registry, Value};

let mut registry = Registry::new();
let package = Package::open("extensions/hello")?;
registry.load_package(&package, &["target/debug".into()])?;
let shout = registry.call("hello", "shout", &[Value::Text("hi".into())])?;
```

Errors, wrong argument counts and panics inside an extension come back
as `CallError`s; the library stays loaded as long as the registry
holds it.

## Build

Requires Rust, [`just`](https://github.com/casey/just), and a checkout
of X_eTaL beside this one (`../X_eTaL`) only when refreshing the
vendored copy.

```sh
just            # list the recipes
just gate       # everything the pre-commit gate checks
just build           # every crate and extension (shared libraries in target/debug/)
just test [CRATE]    # the Rust tests
just test-exts       # every extension's reg-rs tests (reg-rs on PATH)
just videos [EXT]    # record the demos (vhs; window demos from headless frames): nothing on screen
just pages           # build the site into pages/ (the recordings and how to run them); commit it, a push publishes it
just serve-pages     # preview it at http://127.0.0.1:8732/X_eTaL-extensions/
just xetal-version   # which X_eTaL commit is vendored
just eval "'+ r_/ 1 2 3"   # evaluate with the vendored xetal
```

X_eTaL is vendored in `vendor/xetal/` (a snapshot of a committed
X_eTaL commit, named in `vendor/xetal/VENDORED`); `just xetal` builds
its CLI into `target/xetal/`. `just vendor [REF]` refreshes it.

## Development

Development is tracked with agentrail sagas, as in X_eTaL: `agentrail
status` shows the current step, `agentrail next` its instructions.
The plan is `docs/plan.md`; what the extensions need from X_eTaL is
`docs/xetal-asks.md`. Every step ends with the gate passing, docs
updated, a commit to `main` and a push.

## Related Projects

- [X_eTaL](https://github.com/softwarewrighter/X_eTaL) -- the language
  ([try it live](https://softwarewrighter.github.io/X_eTaL/))
- [X_eTaL-libraries](https://github.com/softwarewrighter/X_eTaL-libraries)
  -- libraries written in X_eTaL
- [X_eTaL-demos](https://github.com/softwarewrighter/X_eTaL-demos) --
  visual demos ([live](https://softwarewrighter.github.io/X_eTaL-demos/))
- [sw-mlpl](https://github.com/sw-ml-study/sw-mlpl) -- Software
  Wrighter's Machine Learning Programming Language, a Rust array
  language inspired by APL, APL2, J, and BQN, whose native extension
  design this repository adapts.

## Links

- Blog: [Software Wrighter Lab](https://software-wrighter-lab.github.io/)
- Discord: [Join the community](https://discord.com/invite/Ctzk5uHggZ)
- YouTube: [Software Wrighter](https://www.youtube.com/@SoftwareWrighter)

## Copyright

Copyright (c) 2026 Michael A Wright

## License

MIT. See [`LICENSE`](LICENSE) and [`COPYRIGHT`](COPYRIGHT).
