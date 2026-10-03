# live

Saga 3 of X_eTaL-extensions (docs/plan.md): the extensions running in
the browser -- Rust, Yew and WebAssembly pages published with GitHub
Pages like ../X_eTaL-demos', with the peers' favicon in red. Pages
link extensions statically and run programs with the vendored
xetal-play against a store that serves the facades and the ext:
channel (the same ExtStore as xetal-x).

Rules as before (CLAUDE.md): vendored X_eTaL only, never edited; code
from ../X_eTaL-demos (microscope) and ../../sw-ml-study/demo-extensions
may be copied, never depended on; asks for anything missing; every
step gated, documented, committed with .agentrail/, completed, pushed.

## Steps

1. bridge-lib -- crates/xetal-ext-bridge (ExtStore + protocol + facade
   serving), xetal-x on it; the `static` feature; a native xetal-play
   test of hello's tour matching its golden.
2. shell -- web/shell Yew crate and stylesheet; red favicon.
3. hello-live -- extensions/hello/web: the tour, live and editable.
4. pages -- build-pages.sh, catalog, Pages workflow, Pages enabled.
5. clock-live -- clock for wasm32; extensions/clock/web.
