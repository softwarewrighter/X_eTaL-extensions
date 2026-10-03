# bridge

Saga 2 of X_eTaL-extensions (docs/plan.md): X_eTaL programs calling
native extensions today, with no change to X_eTaL. A bridge host,
`xetal-x`, is the vendored interpreter plus the loader, with a store
that routes `ext:` paths of `[]N_PUT` / `[]N_GET` to native functions
(plan A6; the named workaround for ask E1). Facades hide the channel
behind ordinary typed X_eTaL functions.

Rules as saga 1 (CLAUDE.md): vendored X_eTaL only, never edited;
asks for anything missing; every step gated, documented, committed
with .agentrail/, completed, pushed.

## Steps

1. host-probe -- how the vendored CLI runs a program (crates, entry
   points, the store hook, output); crates/xetal-x running a program
   exactly as `xetal run` does (goldens agree); asks for anything
   missing.
2. bridge -- the ext: store: protocol for arguments (text, numbers
   with shape, two arguments), replies and errors; `xetal-x run --ext
   DIR FILE`; `just run-x FILE`; tests.
3. hello-facade -- Hello.xtl, *.xtl tests with goldens through the
   bridge, pinned facade types, docs/ext/hello.md, `just test-ext NAME`
   in the gate.
4. clock -- the clock extension end to end, a timing demo.
