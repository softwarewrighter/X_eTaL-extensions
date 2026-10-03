# scene

retained 3D line and point scenes in a native window, patched by id

- Package: `extensions/scene/` (`extension.toml`; library
  `xetal_ext_scene`: `libxetal_ext_scene.dylib` on macOS, `.so` on Linux)
- Facade: `lib/Scene.xtl`, recommended alias `sc:`
- Native crates: none beyond the SDK

## Functions

| Native name | Arity | X_eTaL type | What |
| ----------- | ----- | ----------- | ---- |
| `about` | 0 | `Unit -> Char` | the extension's name and version |

## From X_eTaL

```
"sc:" u_se< "Scene"
sc:a_bout @
```

Run with `xetal-x --ext extensions/scene run PROGRAM` (see the
repository's docs/bridge.md).

## Build and test

```sh
just build      # the native library and xetal-x
just test       # Rust tests (rust/tests) and reg-rs tests (tests/)
just list       # the functions as xetal-x sees them
```

## Demos

None yet: `demos/NAME.xtl`, run with `just demo NAME`.
