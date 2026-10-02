# hello

The smallest X_eTaL native extension. It exists to prove the boundary:
each function exercises one thing a real extension needs.

- Package: `extensions/hello/` (`extension.toml`, library
  `xetal_ext_hello`: `libxetal_ext_hello.dylib` on macOS, `.so` on Linux)
- Facade: `Hello.xtl`, recommended alias `hx:` (coming with the bridge)
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

```sh
cargo build -p xetal-ext-hello        # target/debug/libxetal_ext_hello.dylib (or .so)
cargo test -p xetal-ext-hello         # the descriptor and every trampoline, through the raw ABI
cargo test -p xetal-ext-loader        # loaded from its package and linked statically: the same tests
```

When `panic` runs, Rust's default panic hook still prints the panic
message to standard error; the call returns an error and the program
goes on.

The tests call each trampoline exactly as a host does: encode the
arguments, call through the C function pointer, copy the result or
error. A wrong argument count or a malformed value is refused before
the Rust function runs.

## Source

`extensions/hello/src/lib.rs`: eight plain Rust functions and one
`xetal_extension!` listing them with their arity, X_eTaL type and a
line of documentation. No `unsafe`.
