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
`docs/xetal-asks.md`, E1). When X_eTaL gains the hook, the facades
change and the programs do not.

## Extensions

| Extension | Facade | What | Status |
| --------- | ------ | ---- | ------ |
| hello | `Hello` | the smallest proof of the boundary | planned |
| clock | `Clock` | wall-clock and monotonic time, timing | planned |
| digest | `Digest` | SHA-256, CRC-32 | planned |
| regex | `Regex` | match, find, capture, replace, split | planned |
| linalg | `Linalg` | determinant, inverse, solve, least squares, eigenvalues | planned |
| png | `Png` | matrices to and from PNG images | planned |

## Build

Requires Rust, [`just`](https://github.com/casey/just), and a checkout
of X_eTaL beside this one (`../X_eTaL`) only when refreshing the
vendored copy.

```sh
just            # list the recipes
just gate       # everything the pre-commit gate checks
```

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
