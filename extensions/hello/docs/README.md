# hello

The smallest X_eTaL native extension. It exists to prove the boundary:
each function exercises one thing a real extension needs.

- Package: `extensions/hello/` (`extension.toml`; library
  `xetal_ext_hello`: `libxetal_ext_hello.dylib` on macOS, `.so` on Linux)
- Facade: `lib/Hello.xtl`, recommended alias `hx:`
- Native crates: none beyond the SDK

## Functions

| Native name | Arity | X_eTaL type | What it proves |
| ----------- | ----- | ----------- | -------------- |
| `answer` | 0 | `Unit -> Int` | a result with no argument (called with `@`) |
| `add` | 2 | `Num a => a -> a -> a` | two arguments; Int stays Int (overflow is an error), otherwise Float |
| `echo` | 1 | `a -> a` | any value -- scalar, text, array of any rank -- round-trips unchanged |
| `shout` | 1 | `Char -> Char` | text in and out (Unicode upper case) |
| `sum` | 1 | `Num a => a -> Float` | a whole array crossing in one call |
| `kinds` | 1 | `a -> Int` | how the element kinds arrive (Bool, Int, Float, Char counts) |
| `fail` | 0 | `Unit -> Int` | a typed error with a message |
| `panic` | 0 | `Unit -> Int` | a Rust panic contained in the extension, reported as an error; the extension keeps working |

## Build and test

From this directory (or `just ext hello RECIPE` from the repository
root):

```sh
just build      # target/debug/libxetal_ext_hello.dylib (or .so) and xetal-x
just test       # Rust tests (rust/tests: every trampoline through the raw ABI) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
just reg-diff   # reg-rs failures in full; just rebase TEST accepts new output
```

The loader's own tests (`crates/xetal-ext-loader`) load hello from this
package and link it statically, and run the same calls both ways.

When `panic` runs, Rust's default panic hook still prints the panic
message to standard error; the call returns an error and the program
goes on.

The tests call each trampoline exactly as a host does: encode the
arguments, call through the C function pointer, copy the result or
error. A wrong argument count or a malformed value is refused before
the Rust function runs.

## From X_eTaL

```
"hx:" u_se< "Hello"
hx:s_hout "x_etal"                       # X_ETAL
hx:s_um (r_ange 10) '* t_able r_ange 10   # 3025.0
```

Run programs with `xetal-x --ext extensions/hello run PROG.xtl` (or
`just demo NAME` here): `--ext` loads the native library and puts
`lib/` on `XETAL_PATH`. The facade's exports, with their pinned types
(`tests/hello-types.out`):

| Export | Type | Native function |
| ------ | ---- | --------------- |
| `hx:a_nswer @` | `Unit -> Int` | `answer` |
| `a hx:a_dd b` | `(Num a, Num b) => a -> b -> Float` | `add` (sent as Floats, so a Float sum) |
| `hx:s_hout t` | `Char -> Char` | `shout` |
| `hx:s_um x` | `Num a => a -> Float` | `sum` |
| `hx:e_cho x` | `Num a => a -> Float` | `echo` (values and shape come back) |
| `hx:f_ail @` | `Unit -> Int` | `fail`: an X_eTaL error |
| `hx:p_anic @` | `Unit -> Int` | `panic`: contained, an X_eTaL error |

`kinds` has no facade export (its job is showing how element kinds
arrive); `tests/bridge.xtl` calls every native function through the
raw `ext:` channel ([the bridge](../../../docs/bridge.md)), the way
the facade's private helpers do:

```
n := "x_etal" []N_PUT "ext:hello/shout?text"
[]N_GET "ext:hello/shout"                      # X_ETAL
```

A non-numeric argument to `hx:s_um` is a type error before anything is
sent (`tests/facade-type.xtl`).

## Demos

- `demos/tour.xtl` (`just demo tour`): text, numbers, a 10 by 10 times
  table summed in Rust, a rank-3 array out and back with its shape.

## Layout

```
extension.toml   the manifest
justfile         build, test, reg, list, demo
rust/            the crate (src/lib.rs) and its Rust tests
lib/             Hello.xtl, the facade
tests/           reg-rs tests: *.rgt commands and their .out baselines
docs/            this page
demos/           tour.xtl
```

## Source

`rust/src/lib.rs`: eight plain Rust functions and one
`xetal_extension!` listing them with their arity, X_eTaL type and a
line of documentation. No `unsafe`.
