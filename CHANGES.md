# Changes

Every commit, newest first, grouped by day. Times are Pacific
(UTC-07:00), as committed.

Categories: `feat` new capability, `fix` a bug or wrong behavior,
`refactor` structure without behavior change, `test` tests only,
`build` build and tooling, `ext` an extension or a change to one,
`abi` the native ABI, SDK or loader, `docs` documentation, `plan`
saga planning and reordering, `release` milestone release, `chore`
agentrail bookkeeping (step complete, saga archive), `vendor` a
refresh of the vendored X_eTaL.

## 2026-10-03

- 06:28 `feat` hello live: `extensions/hello/web/` (trunk, Yew, the shell): the tour editable and run in the browser with hello compiled to WebAssembly; checked in Chrome (output equal to the command line's, Run works, no console errors); a native test against the golden.

## 2026-10-02

- 06:04 `feat` `web/shell` (`xetal-ext-shell`), the live pages' shared Yew crate, adapted from X_eTaL-demos' microscope: `run::install` links extensions statically and installs the bridge store serving facades and `ext:`; `Playground` (an editable program run in the browser with Run and Reset, its decorated source, output or error, the native functions linked in, the command line); header and footer with the vendored X_eTaL commit; `shell.css` in this repo's red. Native test with hello; `scripts/check-web.sh` in the gate (fmt, clippy, tests, wasm32). The loader re-exports the ABI.

- 19:53 `refactor` `crates/xetal-ext-bridge`: `ExtStore` and the protocol moved out of `xetal-x` (behavior unchanged: gate and goldens), and it serves facade sources by file name for store-only hosts; a native test runs hello's tour through the vendored `xetal-play` with hello linked statically, output identical to the command line's golden. The SDK's entry point is mangled under an extension's `static` feature (hello, clock, the template); the loader's `libloading` sits behind a default `dynamic` feature; hello, clock and the bridge build for wasm32.

- 19:35 `plan` Saga 3, live (the user's request): the extensions in the browser with Rust, Yew and WebAssembly, published with GitHub Pages like X_eTaL-demos; steps bridge-lib, shell, hello-live, pages, clock-live; later sagas renumbered 4-8. `images/favicon.ico`: the peers' favicon with its lavender ground turned red. Saga 2 archived.

- 19:20 `ext` The clock extension (`extensions/clock`, alias `ck:`): now, iso, iso_of (Hinnant civil dates, years 0000-9999), millis (monotonic), and `ck:t_ime` in the facade; Rust tests of dates, 5 reg-rs tests; `demos/bridge-cost.xtl` measures the bridge (about 220,000 calls/s, 2.6 M Floats/s out and back, release); docs/bridge.md cost section; ask E4 (a clock quad).

- 18:58 `ext` The Hello facade (`extensions/hello/lib/Hello.xtl`, alias `hx:`): a_nswer, a_dd, s_hout, s_um, e_cho, f_ail, p_anic over private ext: helpers, typed like the native signatures (a wrong argument is a type error before any call); `xetal-x` puts each loaded package's `lib/` on `XETAL_PATH`; reg-rs tests of the facade, its pinned types, a type error, a failure, and `demos/tour.xtl`; hello's page complete; the template's facade carries the helpers.

- 18:39 `feat` The bridge protocol: `xetal-x`'s `ext:` store takes arguments as `[]N_PUT` to `ext:E/F?KIND=SHAPE` (text, chars, int, float, bool), calls on `[]N_GET "ext:E/F"`, and gives the reply's `?shape` and `?kind`; every failure is an X_eTaL `error[io]` naming the path; floats cross in shortest round-trip form. The SDK keeps contained panics quiet. hello's reg-rs tests: every function through the channel, failure, panic, arity, a bad argument. docs/bridge.md.

- 17:06 `plan` The catalog and sagas chosen with the user: four flagship demos (data notebook on sqlite; X_eTaL on the web with axum, a live page and TodoMVC; photo lab with image and linalg; fetch and analyze with http), supporting clock and digest, hello as the teaching example; sagas 3-6, the blocked native-hook saga now 7; A15 (path-named state, axum behind a pull interface, loopback-only network tests).
- 17:06 `build` Each extension a self-contained directory (plan A7, the user's rule): hello moved to `extensions/hello/{extension.toml, justfile, rust/, lib/, tests/, docs/, demos/}`; reg-rs per extension (`scripts/reg-ext.sh`, baselines in its `tests/`) with hello's first test; root `just ext NAME RECIPE`, `test-exts` (in the gate, also checking the layout), `new-ext` from `templates/extension/`.

- 15:36 `feat` The bridge host `crates/xetal-x`: the vendored CLI's modules compiled unchanged (`#[path]`, rustfmt-skipped) with its `main` repeated and the store wrapped in `ExtStore`, which routes `ext:EXT/FN` paths to loaded extensions (calls wired next step; unknown extensions and functions are X_eTaL `error[io]`s). `--ext DIR` / `XETAL_EXT_PATH` load packages; `--ext-list`. `scripts/check-xetal-x.sh` (all 20 vendored demos identical under xetal and xetal-x) and a vendor-untouched check in the gate; 5 host tests; `just run-x`, `ext-list`, `check-xetal-x`; ask E3 (the CLI as a library).

- 15:02 `docs` The X_eTaL logo (the corrected 2026-10-02 version) at the top of the README, `images/modern-xetal-logo.jpg`.

- 13:11 `plan` Saga 2, bridge: `xetal-x` (the vendored interpreter, the loader and an `ext:` store) so X_eTaL programs call native extensions today; steps host-probe, bridge, hello-facade, clock. Saga 1 archived.

- 13:04 `chore` Saga foundation completed (step loader).
- 13:04 `abi` Loader: `crates/xetal-ext-loader` (adapted from demo-extensions): `Package` reads `extension.toml` (name, version, abi, facade, library stem; unknown keys and path-like names refused) and finds the library under `native/<platform>/` or a build directory; `Registry` loads with `libloading` (or links statically), validates, checks the descriptor against the manifest, refuses duplicates, checks arity before calling, copies results and errors, deactivates, gives `help`. 9 tests run hello both ways. `just build`; README "Loading an extension from Rust"; plan saga 1 retrospective.

- 12:39 `chore` Saga step sdk-hello completed.
- 12:39 `abi` SDK and the hello extension: `crates/xetal-ext-sdk` (`xetal_extension!` generates the descriptor, the unmangled `xetal_extension_v1` entry and a panic-safe trampoline per function, which checks the argument count and decodes arguments before the handler runs; argument helpers `text`, `number`, `float_vector`); `extensions/hello` (cdylib + rlib, `extension.toml`): answer, add, echo, shout, sum, kinds, fail, panic, with 5 tests calling every trampoline through the raw ABI; docs/ext/hello.md; README "Writing an extension". The descriptor constructor is now `const`.

- 12:29 `chore` Saga step abi completed.
- 12:29 `abi` ABI V1: the root Cargo workspace and `crates/xetal-ext-abi` (adapted from demo-extensions): the `xetal_extension_v1` descriptor (size and version first; functions with arity 0-2, X_eTaL signature, doc, trampoline), tagged values (Bool, Int, Float, Text, dense Bool/Int/Float/Char arrays of rank 0-9), errors, validation and value decoding that check every bound and copy into owned values, panic containment; 11 contract tests (pinned layout, every rejection, round trips). Gate adds `scripts/check-rust.sh` (fmt, clippy -D warnings, tests); docs/abi-v1.md.

- 12:09 `chore` Saga step vendor-xetal completed.
- 12:09 `build` Vendoring: `just vendor [REF]` snapshots a committed ref of ../X_eTaL into vendor/xetal/ (VENDORED records it); `just xetal`, `xetal-version`, `eval`, `check-vendor` (the CLI builds, answers, names the vendored commit, runs life.xtl, imports Stats); in the gate. Scripts as in ../X_eTaL-libraries.
- 12:05 `vendor` X_eTaL 7b70575 vendored.

- 11:15 `chore` Saga step scaffold completed.
- 11:15 `plan` Scaffold: the agentrail process (saga foundation), CLAUDE.md/AGENTS.md, README, COPYRIGHT, LICENSE, CHANGES.md, justfile, the gate, docs/plan.md (architecture A1-A14: vendored X_eTaL, ABI V1, SDK, loader, the `ext:` bridge host until X_eTaL has a native hook, facades; the catalog of 8 extensions; four sagas), docs/xetal-asks.md (E1 native hook, E2 `.xtlm`), docs/research.txt (archival copy).

- 08:47 `chore` First commit: an empty README.
