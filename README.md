<p align="center">
  <img src="images/modern-xetal-logo.jpg" alt="X_eTaL: X underlined, a raised e, T, a raised a, L" width="360">
</p>

# X_eTaL extensions

Native extensions for [X_eTaL](https://github.com/softwarewrighter/X_eTaL),
the eXperimental Extensible Typed Array Language: small Rust libraries
that give X_eTaL programs what the interpreter cannot do by itself --
a clock, hashing, regular expressions, fast linear algebra, image
files -- each used from X_eTaL like any other library.

## What this is

X_eTaL is extensible on two axes: what programs can *say* (macro
libraries, see
[X_eTaL-libraries](https://github.com/softwarewrighter/X_eTaL-libraries))
and what programs can *do*. This repository is the second: native
code behind an ordinary X_eTaL facade.

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
- a facade library (`Digest.xtl`) gives each function an X_eTaL name
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

Four demos are planned, each pairing an X_eTaL array program with a
real Rust crate:

| Demo | Extensions | What it shows | Status |
| ---- | ---------- | ------------- | ------ |
| data notebook | sqlite | a CSV in SQLite; SQL selects, arrays compute group-bys, histograms, a fit; an SVG chart | planned |
| X_eTaL on the web | web, sqlite | a live page recomputing Life or Mandelbrot per request; a TodoMVC stored in SQLite | planned |
| photo lab | image, linalg | a photo as an array: filters, edges, SVD compression; PNGs out | planned |
| fetch and analyze | http | live earthquake data summarized and plotted | planned |

| Extension | Facade | What | Crates | Status |
| --------- | ------ | ---- | ------ | ------ |
| [hello](extensions/hello/docs/README.md) | `Hello` | the smallest proof of the boundary | -- | done: facade, tests, demo |
| [clock](extensions/clock/docs/README.md) | `Clock` | wall-clock and monotonic time; the bridge's cost | std | done: facade, tests, cost demo |
| sqlite | `Sqlite` | execute and query SQLite files; CSV import | rusqlite | planned |
| web | `Web` | serve HTTP: the program takes each request and replies | axum, tokio | planned |
| image | `Image` | images to and from arrays | image | planned |
| linalg | `Linalg` | solve, inverse, least squares, eigenvalues, SVD | nalgebra | planned |
| http | `Http` | bounded GET | ureq | planned |
| digest | `Digest` | SHA-256, CRC-32 | sha2, crc32fast | planned |

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
