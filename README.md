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
"dg:" u_se< "Digest"
dg:s_ha256 "abc"
```

The program does not know whether `dg:s_ha256` is X_eTaL, Rust or C.
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
``` When X_eTaL gains the hook, the facades
change and the programs do not.

## Extensions

| Extension | Facade | What | Status |
| --------- | ------ | ---- | ------ |
| [hello](docs/ext/hello.md) | `Hello` | the smallest proof of the boundary | native library built, loaded and tested; facade waits for the bridge |
| clock | `Clock` | wall-clock and monotonic time, timing | planned |
| digest | `Digest` | SHA-256, CRC-32 | planned |
| regex | `Regex` | match, find, capture, replace, split | planned |
| linalg | `Linalg` | determinant, inverse, solve, least squares, eigenvalues | planned |
| png | `Png` | matrices to and from PNG images | planned |

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

See [`extensions/hello`](extensions/hello) and the
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
