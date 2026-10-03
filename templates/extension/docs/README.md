# __NAME__

__WHAT__

- Package: `extensions/__NAME__/` (`extension.toml`; library
  `__STEM__`: `lib__STEM__.dylib` on macOS, `.so` on Linux)
- Facade: `lib/__CAP__.xtl`, recommended alias `__ALIAS__:`
- Native crates: none beyond the SDK

## Functions

| Native name | Arity | X_eTaL type | What |
| ----------- | ----- | ----------- | ---- |
| `about` | 0 | `Unit -> Char` | the extension's name and version |

## From X_eTaL

```
"__ALIAS__:" u_se< "__CAP__"
__ALIAS__:a_bout @
```

Run with `xetal-x --ext extensions/__NAME__ run PROGRAM` (see the
repository's docs/bridge.md).

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (rust/tests) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
```

## Demos

None yet: `demos/NAME.xtl`, run with `just demo NAME`.
