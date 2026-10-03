# X_eTaL-extensions -- Implementation Plan

Native extensions for X_eTaL (the eXperimental Extensible Typed Array
Language, developed in `../X_eTaL`): independently built Rust
libraries (`.dylib` / `.so`, C ABI) that add capabilities the
interpreter does not have -- hashing, regular expressions, clocks,
fast linear algebra, image files -- each wrapped in an ordinary `.xtl`
facade library so a program uses it like any other library, and,
once X_eTaL supports them, `.xtlm` macro helpers that generate those
facades. The ideas come from `docs/research.txt` (archival, not
normative; a copy of `../X_eTaL-libraries/docs/research.txt`); the
architecture follows `../../sw-ml-study/demo-extensions`, which did
the same for sw-MLPL; the process follows `../X_eTaL-demos`,
`../X_eTaL-games` and `../X_eTaL-libraries`.

Development is driven by agentrail sagas (one active saga in
`.agentrail/`, finished sagas archived to `.agentrail-archive/`).
Every step ends with the gate (`just gate`), docs updated
(`README.md`, `CHANGES.md`, this plan, `docs/xetal-asks.md`, the
extension's page), a sane `.gitignore`, a detailed commit to `main`
(with the `.agentrail/` changes), `agentrail complete`, and a push.

## Guiding principle

X_eTaL's "Extensible" has two axes (research.txt):

| Axis | Mechanism | Lives in |
| ---- | --------- | -------- |
| what programs can **do** | native code behind an X_eTaL facade: a Rust `cdylib` exporting one C-ABI descriptor, loaded by the host, called from `.xtl` | **this repo** |
| what programs can **say** | `.xtlm` macro libraries, `(String, String) -> String` run before parsing | `../X_eTaL-libraries` (samples); here only macro helpers for bindings, once X_eTaL has `.xtlm` |

research.txt's rule for the native axis: the FFI primitive (`[]S_VO`)
is an implementation boundary, not something application code should
see. A program writes `"dg:" u_se< "Digest"` and `dg:s_ha256 "abc"`;
whether `dg:s_ha256` is X_eTaL, Rust or C is the facade's business.

An extension earns its place when it does something X_eTaL cannot do
at all (the clock, regular expressions, binary files) or cannot do
fast enough (hashing, dense linear algebra), and it is small enough
to read in one sitting.

## Where X_eTaL is (vendored era)

- No FFI. `[]S_VO` is reserved for "channels to special facilities
  (graphics, a Rust dynamic library)" (lang-choices QD4, section 15;
  X_eTaL plan "Later -- system I/O"), not designed or built.
- `.xtlm` macro libraries are decided (MC10-MC13) and planned as
  X_eTaL Saga 19, not built.
- The one host hook is the **store** (`xetal_store::install(Arc<dyn
  Store>)`): the host decides what `[]N_GET path`, `t []N_PUT path`,
  `[]R_EAD @` and `[]S_HOW svg` do (the browser maps them to local
  storage and a pane). That is enough for a text channel -- exactly
  APL's shared-variable model, after which `[]S_VO` is named.

So extensions are built, loaded and tested from Rust today; programs
reach them through a bridge host (decision A6) until X_eTaL has a
native hook (ask E1), and then the facades switch with no change to
programs.

## Architecture decisions

| # | Decision | Why |
| - | -------- | --- |
| A1 | X_eTaL is **vendored** into `vendor/xetal/` as a source snapshot of a committed ref of `../X_eTaL` (`just vendor [REF]`, default `HEAD`), recorded in `vendor/xetal/VENDORED`; the CLI builds into `target/xetal/` (`just xetal`). Same scripts as the sibling repos. Never edited; refreshed at a saga start or when an ask lands, in its own commit. | X_eTaL moves fast; extensions need a recent but stable interpreter. |
| A2 | **One Cargo workspace** at the root for this repo's crates (`crates/*`, `extensions/*/rust`), separate from the vendored X_eTaL's own workspaces; everything builds into `./target` (`.cargo/config.toml`). | One `cargo test` covers the ABI, SDK, loader and every extension. |
| A3 | **ABI V1** (`crates/xetal-ext-abi`, `docs/abi-v1.md`): every extension exports one symbol, `xetal_extension_v1`, returning a `#[repr(C)]` descriptor: size and ABI version first, name and version (UTF-8 slices), a bounded table of functions (name, arity 0 to 2, the X_eTaL type signature such as `Char -> Char`, a one-line doc, the trampoline), a reserved field that must be zero. Values carry a fixed-width tag and a payload: X_eTaL's scalars (Bool, Int as i64, Float as f64), text (UTF-8, a Char vector) and dense row-major arrays of Bool, Int, Float or Char of rank 0 to 9; an error is a code and a message. Bounded: 64 Mi elements per value, 16 KiB of descriptor text, 1,024 functions. | demo-extensions' ABI V1, reduced to X_eTaL's types; the size/version prefix and zero reserved fields reject layout drift; the signature lets a future native hook (E1) type the function without a facade. |
| A4 | **Safety at the boundary.** Validation is the only unsafe pointer-reading code; it copies everything into owned Rust values before registration. Rust panics inside an extension are caught in the extension (SDK trampolines) and returned as an error status, never unwound across `extern "C"`. A result lives in storage the extension keeps until its next call on the same thread; the host copies it at once. | Unwinding across C is undefined; a loader that keeps foreign pointers is a use-after-free waiting to happen. |
| A5 | **SDK and loader.** `crates/xetal-ext-sdk`: what an author writes (a `xetal_extension!` macro over plain Rust functions of `&[Value] -> Result<Value, String>`), generating the descriptor and the panic-safe trampolines, keeping each result until the next call. `crates/xetal-ext-loader`: resolves a package, `libloading`s it, validates, and registers its functions under the extension's namespace, checking arity before every call, keeping the `Library` alive as long as any function. The same descriptor can also be linked statically (tests, the browser later). | The author never writes `unsafe`; the host never trusts the extension. |
| A6 | **A bridge host until X_eTaL has a native hook.** `crates/xetal-x`: the vendored X_eTaL interpreter plus the loader, installing a store that routes paths starting `ext:` to native functions and everything else to the disk: `request []N_PUT "ext:digest/sha256"` calls the function on the request text and keeps the reply; `[]N_GET "ext:digest/sha256"` takes it. It is a documented workaround (ask E1), text-only (numbers cross as `f_ormat` / `n_umbers` text), and every facade hides it behind ordinary functions so programs never see it. Programs run with `xetal-x run FILE` instead of `xetal run FILE`. | Lets every extension be used end to end from X_eTaL today with no change to X_eTaL; it is APL's shared-variable channel, the model `[]S_VO` names. Replaced when E1 lands. |
| A7 | **An extension is a self-contained directory**, `extensions/<name>/` (the user's rule, 2026-10-02): `extension.toml` (the manifest); `justfile` (its own recipes: build, test, reg, demos); `rust/` (its Rust library crate, a `cdylib` + `rlib`, with its Rust tests); `lib/` (its X_eTaL sources: the facade `<Name>.xtl`, and `<Name>.xtlm` macros when X_eTaL has them -- one directory, as X_eTaL looks for both together, MC11); `tests/` (reg-rs tests: `*.rgt` commands with committed `.out`/`.err` baselines, `REG_RS_DATA_DIR` pointing there, and the `*.xtl` programs they run); `docs/` (its pages, `docs/README.md` first); `demos/` (programs that show it off, each with a README section and a reg-rs test). The repo's `just` recipes delegate to each extension's. | Each extension can be read, built, tested and moved on its own; the layout is the same for every one. |
| A8 | **Facade conventions** follow X_eTaL's style guide (lang-choices section 16) and `../X_eTaL-libraries` A6: `UpperCamel.xtl`, exports under `l:`, private helpers unprefixed, predicates `?`, effects `!`, a header with the import line and recommended alias, no export shadowing a built-in, no name shadowing a standard library. Exports are typed as if native (ordinary X_eTaL types), so the facade's types are pinned (`xetal type`) and survive the switch from the bridge to E1. | The facade is the extension's interface; its types are the contract. |
| A9 | **Pure fallbacks where cheap.** When an extension's function has a reasonable pure X_eTaL definition (CRC-32, a small determinant), the facade's tests compare native and pure results; the pure version is not exported. Where there is none (the clock, regular expressions), there is no fallback. | Golden behavior is cross-checked, not only self-consistent. |
| A10 | **Macro helpers wait for X_eTaL** (ask E2, X_eTaL MC10-MC13): a `.xtlm` that turns a signature into a facade function (`"f64 f64" ffi:b_ind< "linalg/det"`). Designed on paper only (saga 9), never emulated. | As in `../X_eTaL-libraries` A9. |
| A11 | A missing X_eTaL feature or bug an extension uncovers is **not** fixed here nor hidden: it goes in `docs/xetal-asks.md` (status, kind, extensions, why, minimal repro, workaround) and on the extension's page. | X_eTaL owns its language decisions; this repo is a consumer. |
| A12 | **Names, not homes**: an extension is identified by its name (`digest`), never a GitHub coordinate (the repos may move to `sw-array-languages`, research.txt). `just` is the entry point (recipes call `scripts/*.sh`); `CHANGES.md` gets a line for every commit; docs are ASCII-only markdown (`sw-markdown-checker`). | Same process as the sibling repos. |
| A13 | Dependencies are few, well known and permissively licensed (`libloading`, and per extension e.g. `rusqlite` with bundled SQLite, `axum`/`tokio`, `ureq`, `image`, `nalgebra`); each extension's page lists its crates. Nothing is downloaded at run time except by the http extension, on request. | Small, auditable extensions. |
| A15 | **Practical extensions, state named by path.** ABI V1 has no handles, so state lives where a path names it (an SQLite file per call, opened and closed by the extension) or inside the extension (a server's request queue). Handles (tag 7, reserved) are added only when a demo needs them. The web extension keeps axum's async world inside the extension: a background thread runs the server and queues requests; X_eTaL pulls the next request and posts its reply (callback-free, as demo-extensions' http-server), so the program drives. Network tests use loopback only; live fetches are opt-in recipes. | The user's choices (2026-10-02): practical demos, axum, loopback tests, path-based state. |
| A14 | **Copied from demo-extensions, never dependent on it.** Design, docs and code from `../../sw-ml-study/demo-extensions` (same author, MIT) may be copied and adapted to X_eTaL's types, with the source file noted in a comment; no Cargo path, git or build dependency on that repo or on sw-MLPL. | Reuse proven work while this repo builds and moves on its own. |

## Layout

```
crates/
  xetal-ext-abi/         the C ABI V1: layout, values, validation
  xetal-ext-sdk/         what extension authors use (xetal_extension!)
  xetal-ext-loader/      packages, dynamic loading, registry
  xetal-ext-bridge/      the ext: channel: ExtStore and the protocol, for any host
  xetal-x/               the bridge host: vendored xetal + loader + ExtStore
web/shell/               the live pages' shared Yew crate and stylesheet
images/                  the logo and the red favicon
extensions/<name>/       one self-contained directory per extension
  extension.toml         the package manifest
  justfile               its recipes (build, test, reg, demo)
  rust/                  its Rust library crate (cdylib + rlib) and Rust tests
  lib/                   <Name>.xtl, the facade (and <Name>.xtlm, later)
  tests/                 reg-rs tests: *.rgt, .out/.err baselines, *.xtl programs
  docs/                  README.md (its page), and any further pages
  demos/                 programs showing it off
  web/                   its live page (a Yew app; its own Cargo workspace)
templates/extension/     what just new-ext copies
docs/plan.md             this plan
docs/xetal-asks.md       what the extensions need from X_eTaL
vendor/xetal/            the vendored X_eTaL (never edited)
scripts/                 the logic behind the just recipes
```

## The catalog

Release 1 (research3.txt in ../X_eTaL, "Do SQLite", accepted
2026-10-03): **hello + clock + sqlite**, with the data notebook as the
flagship demo -- one serious extension that proves the boundary on
something X_eTaL should not reinvent, rather than many half-done ones.
Everything else below is roadmap, labelled so in the README, and waits
until after release 1.

Chosen with the user (2026-10-02): four flagship demos built on
practical crates, a few small supporting extensions, and hello as the
teaching example. Aliases are recommendations (the importer chooses,
MC3), distinct from the standard libraries' (`c:`, `m:`, `s:`) and
`../X_eTaL-libraries`'.

Flagship demos:

| Demo | Extensions | What it shows |
| ---- | ---------- | ------------- |
| data notebook | sqlite | a CSV loaded into SQLite, SQL selects, X_eTaL arrays compute group-bys, histograms and a linear fit, an SVG chart (`[]G_RID`); SQL and arrays complement each other |
| X_eTaL on the web | web, sqlite | a live page recomputing Life or Mandelbrot as SVG on each request; a TodoMVC persisted in SQLite; the X_eTaL program is the request loop |
| photo lab | image, linalg | a photo as an n by m by 3 array: filters and edges by rotation, PNG out; SVD compression at several ranks |
| fetch and analyze | http | live data (the USGS earthquake CSV feed, opt-in) summarized and plotted; tests fetch from the web extension on loopback |

Extensions:

| Extension | Facade, alias | What | Native crates | Saga |
| --------- | ------------- | ---- | ------------- | ---- |
| hello | `Hello`, `hx:` | the smallest proof of the boundary: an answer, add, echo, shout, sum, kinds, a typed error, a contained panic | none | 1-2 |
| clock | `Clock`, `ck:` | wall-clock time (Unix seconds, ISO 8601 text), a monotonic millisecond counter; measures the bridge's cost | std | 2 |
| sqlite | `Sqlite`, `sq:` | `db sq:e_xec sql` (statements, changed-row count), `db sq:q_uery sql` (a result as a Char matrix of cells, or numeric columns), parameters bound from a vector, CSV import; files confined to the directories given | `rusqlite` (bundled) | 3 |
| web | `Web`, `wb:` | `wb:s_erve! port` (axum on a background thread, loopback by default), `wb:n_ext! @` (the next request: method, path, query, body), `wb:r_eply! response` (status, content type, body); static files from a directory | `axum`, `tokio` | 4 |
| image | `Image`, `im:` | read PNG/JPEG as an n by m by 3 (or n by m) Int array, write one as PNG, resize | `image` | 5 |
| linalg | `Linalg`, `la:` | determinant, inverse, solve, least squares, symmetric eigenvalues, SVD on Float matrices | `nalgebra` | 5 |
| http | `Http`, `ht:` | bounded GET (size, time, redirects limited) of text | `ureq` | 6 |
| digest | `Digest`, `dg:` | SHA-256 and CRC-32 (supporting: content checks of fetched data) | `sha2`, `crc32fast` | 6 |
| regex | `Regex`, `rx:` | match?, find, captures, replace, split (supporting: parsing text) | `regex` | later |

Held back: a native window for live animation (macOS wants windows on
the main thread, which the bridge does not give) and a tokenizer
(closer to the ML line, OMLETA, than to X_eTaL).

## Saga 1 -- foundation

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | scaffold | the agentrail saga; CLAUDE.md (AGENTS.md a symlink); README; COPYRIGHT; LICENSE; CHANGES.md; .gitignore; justfile; the gate (markdown); this plan; docs/xetal-asks.md; docs/research.txt (archival copy) |
| 2 | vendor-xetal | `just vendor [REF]`, `vendor/xetal/VENDORED`, `just xetal`, `xetal-version`, `eval`; `scripts/check-vendor.sh` in the gate; the snapshot in its own commit |
| 3 | abi | the root workspace; `crates/xetal-ext-abi`: descriptor, values, errors, validation and its tests (layout drift, bounds, bad UTF-8, duplicates, version mismatch); `cargo test`, `fmt`, `clippy` in the gate; docs/abi-v1.md (done) |
| 4 | sdk-hello | `crates/xetal-ext-sdk` (`xetal_extension!`, panic-safe trampolines, results kept until the next call); `extensions/hello` as a cdylib with its manifest; Rust tests of the descriptor (done) |
| 5 | loader | `crates/xetal-ext-loader`: manifests, the platform's artifact, `libloading`, validation, the registry (namespaces, arity, lifetime); hello loaded dynamically and statically, the same tests on both (done) |

Saga 1 retrospective: the boundary works end to end from Rust (25
tests: ABI contract, hello through raw trampolines, hello loaded from
its package and statically). Changes from the first plan: results are
kept in the extension until its next call (demo-extensions' model)
rather than freed by a callback; descriptors carry X_eTaL signatures
and arity is 0 to 2, as X_eTaL calls functions; a package's manifest is
`[extension]` with `name`, `version`, `abi`, `facade`, `library` (the
file stem), the library found under `native/<platform>/` or a build
directory.

## Saga 2 -- the bridge

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | host-probe | how a host runs a program with the vendored crates (the CLI's run path, the store hook); `crates/xetal-x` skeleton that runs a program exactly as `xetal run` does (goldens agree); asks for anything missing (done: the vendored CLI's modules compiled as `#[path]` modules, its `main` repeated with `ExtStore`; all 20 vendored demos identical, `scripts/check-xetal-x.sh` in the gate; ask E3) |
| 2 | layout | each extension a self-contained directory (A7): hello moved to `extensions/hello/{extension.toml,justfile,rust/,lib/,tests/,docs/,demos/}`; reg-rs wired (`scripts/reg-ext.sh`, per-extension `tests/` as `REG_RS_DATA_DIR`, `.tdb*` ignored); the root `just ext NAME RECIPE` and `just test-ext` delegating; `templates/extension/` and `just new-ext NAME`; the gate runs every extension's tests; this plan's new catalog and sagas |
| 3 | bridge | the `ext:` store (A6): the text protocol for arguments (text, numbers with shape, two arguments), replies and errors; reg-rs tests calling hello (done: docs/bridge.md; `?text`, `?chars=S`, `?int=S`, `?float=S`, `?bool=S` puts, the call as a get, `?shape` and `?kind`; the SDK keeps contained panics quiet) |
| 4 | hello-facade | `lib/Hello.xtl`, its reg-rs tests through the bridge, pinned facade types, a demo, hello's docs complete (done; `xetal-x` puts each package's `lib/` on `XETAL_PATH`; facade helpers apply `f_loat` so facade types match the native signatures; the template's facade carries the helpers) |
| 5 | clock | the clock extension end to end; the bridge's cost measured (calls per second, bytes per second) (done: now, iso, iso_of, millis; `ck:t_ime`; about 220,000 calls/s and 2.6 M Floats/s out and back; ask E4) |

## Saga 3 -- live

The extensions running in the browser (the user's request,
2026-10-02): Rust, Yew and WebAssembly pages published with GitHub
Pages, as `../X_eTaL-demos` publishes its demos, with the peers'
favicon in red (`images/favicon.ico`). A browser cannot load a shared
library, so a page links its extensions statically; the vendored
`xetal-play` runs programs against the store the page installs, and the
same `ExtStore` that `xetal-x` uses serves the facades (`u_se<` reads
`Name.xtl` from the store) and the `ext:` channel. Extensions whose
crates build for `wasm32` go live (hello, clock, digest, image,
linalg); sqlite, web and http stay native.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | bridge-lib | (done) `crates/xetal-ext-bridge`: `ExtStore` and the protocol out of `xetal-x`, generic over the inner store, also serving each extension's facade by name; a `static` feature on every extension (no unmangled entry, so several link into one binary; never enabled inside the root workspace); a native test running hello's tour through `xetal-play` with hello linked statically, matching its reg-rs golden; the loader's `libloading` behind a default `dynamic` feature, off for the browser |
| 2 | shell | (done) `web/shell`, the pages' shared Yew crate (adapted from X_eTaL-demos' microscope, copied not depended on): header with the logo, an editable program panel run in the browser, the output, decorated source, footer naming the vendored X_eTaL commit; its stylesheet; the red favicon |
| 3 | hello-live | (done) `extensions/hello/web/`: the tour, editable and run live; native tests of the page's model |
| 4 | pages | (done: https://softwarewrighter.github.io/X_eTaL-extensions/) `scripts/build-pages.sh` (trunk into `pages/NAME/`, a catalog `pages/index.html`), `.github/workflows/pages.yml` (publishes `pages/`), GitHub Pages enabled, README link |
| 5 | clock-live | (done) clock for `wasm32` (the browser's clock through `js-sys`), `extensions/clock/web/`: the bridge-cost demo live |

Saga 3 retrospective: hello and clock are live at
https://softwarewrighter.github.io/X_eTaL-extensions/ -- programs
editable and run in the browser, the extensions compiled to
WebAssembly and linked statically (the `static` feature lets two link
into one page), the same `ExtStore` serving facades and `ext:` in the
browser as on the command line. Found on the way: page crates must
build into their own `target/web/` (sharing `deps/` with the root
workspace mixed two builds of the ABI crate); std's clocks panic on
`wasm32`, so clock reads the browser's. In a browser the channel is
about ten times slower than natively (about 16,000 calls/s), which a
native hook (E1) would remove.

## Saga 4 -- sqlite, the data notebook, release 1

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | sqlite | (done) the sqlite extension: exec, query (cells as a Char matrix, numeric columns as Float), parameters, confinement to allowed directories, errors; Rust tests and reg-rs tests on a temp database |
| 2 | csv-import | (done) CSV into a table (native, `rusqlite` + a small CSV reader), and query results back as X_eTaL arrays |
| 3 | notebook | (done; NOAA/Scripps Mauna Loa CO2) demos/notebook: a bundled CSV (a public-domain dataset), SQL plus array analytics, an SVG chart; reg-rs golden |
| 4 | sqlite-live | (probed: feasible with rusqlite 0.40 + `sqlite-wasm-rs`, but it needs LLVM's clang to compile SQLite for wasm32 -- Apple's clang cannot. The user's decision, 2026-10-03: no SQLite in the browser; sqlite stays command-line only. Browser persistence (the browser's own storage) or a client/server split -- X_eTaL with extensions on a server, a WebAssembly X_eTaL without extensions as the browser UI, talking CRUD REST or WebSockets -- are possible later demos, but not extensions as such; the web saga is the natural home for the server side) whether SQLite runs in the browser (rusqlite on wasm32 via sqlite-wasm-rs or similar); if so with modest work, the notebook live; if not, why, and the catalog lists sqlite as command-line only |
| 5 | release-1 | (done) research3's checklist for this repo: README value proposition and the three-ways phrase ("libraries extend the vocabulary; macros extend the language; native extensions extend the machine"); roadmap extensions labelled; `docs/status.md`, what works today (this repo's column of the ecosystem dashboard, generated from manifests and tests where possible); stale-ask sweep against upstream (vendor refreshed if asks landed); the site's catalog lists every extension; a fresh-user walkthrough (clone, build, hello, clock, the sqlite notebook) as a recipe, run and fixed; retrospective; a release tag proposed to the user |

Saga 4 retrospective (release 1): sqlite is the serious extension
research3 asked for -- statements, numeric and text results, quoting,
confined paths, CSV import -- and the data notebook shows SQL and
X_eTaL each doing what it is good at on real data (Mauna Loa CO2).
Release 1 is hello + clock + sqlite: `docs/status.md` (generated,
checked by the gate) says what works today; the site's catalog lists
all three; a fresh clone builds with nothing but Rust and runs release
1's programs against their goldens (`just walkthrough`). Found on the
way: asks E5 (errors name the calling line) and E6 (`--draw` before
the subcommand); SQLite in the browser needs LLVM's clang. Upstream,
E1 is X_eTaL Saga 23 and E2 Saga 19; none of our asks has landed yet.

## Saga 5 -- media: the MP3 visualizer

The user's request (2026-10-03): demos like demo-extensions' MP3 player
visualizer (docs/parity.md). Rust decodes and plays the audio and
renders 3D; X_eTaL analyses each chunk and builds the scene as arrays.

Decisions:

- M1. The window belongs to the host. On macOS a window must live on
  the main thread, and X_eTaL runs a program on a worker thread. So
  `xetal-x` runs the CLI on a thread of its own and gives the main
  thread to a UI service (`crates/xetal-ext-ui`); a UI extension is
  linked statically into `xetal-x` and talks to the service through
  queues. A program pulls the next event (a frame tick, a key, close)
  and pushes changes: callback-free, as the web extension will be.
- M2. Retained scenes with stable ids, adapted (copied) from
  demo-extensions' `mlpl-native3d-scene` and `-window` (winit, wgpu):
  a program sends a polyline or points under an id once and patches
  it later; the camera orbits under the mouse.
- M3. Audio state lives in the extension under an Int id (a handle by
  convention, no ABI change): open a file, read bounded chunks for
  analysis, play on the device from a decode-ahead queue independent
  of the program's frame rate (Symphonia, CPAL).
- M4. A program loops with `p_ower` over a bounded number of frames;
  frames after quitting do nothing. (Tail calls or a while loop would
  be cleaner: asked upstream if needed.)
- M5. Test media is generated (tones by sox or ffmpeg, committed small,
  ours); personal media goes in an ignored `local-media/`.
- M6. Ogg Vorbis first (the user's preference, 2026-10-03): fixtures
  and the demo's default media are Ogg Vorbis (royalty-free by
  design); MP3 is supported too -- its patents expired around 2017 and
  Symphonia's decoder (pure Rust, MPL-2.0) needs no licence.
- M7. Drawing: the 2D canvas (the UI host's first extension) uses
  softbuffer (a CPU pixel buffer in a winit window); the 3D scene uses
  wgpu as demo-extensions does. winit 0.30 for windows and events in
  both.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | parity | docs/parity.md; this saga |
| 2 | ui-host | (done) `crates/xetal-ext-ui` and `xetal-x` running the program off the main thread; a minimal window extension (an array shown as pixels, events back) proving the loop; headless tests of the queues |
| 3 | scene3d | the scene extension: retained lines and points by id, colours, camera, events; a wireframe cube demo |
| 4 | audio | the audio extension: open, info, chunks (2 by n Floats), play, pause, seek, position, close; tests on generated MP3 and Ogg fixtures |
| 5 | spectrum | demos/spectrum: the visualizer in X_eTaL (spectrum by inner product, radial spokes, keys); headless tests of the analysis on fixtures; an interactive smoke |
| 6 | media-release | docs, parity updated, status, retrospective |

## Saga 6 -- web (roadmap, after release 1)

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | web | the web extension: axum on a background thread, a bounded request queue, next / reply, timeouts, loopback by default; Rust tests with a real client on loopback |
| 2 | live-page | demos/live: a page recomputing Life or Mandelbrot as SVG per request; reg-rs test drives it over loopback |
| 3 | todomvc | demos/todomvc: TodoMVC in X_eTaL with sqlite; loopback test of add, toggle, delete |

## Saga 7 -- photo lab (roadmap)

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | image | the image extension: read, write, resize; test images generated, not downloaded |
| 2 | linalg | the linalg extension; small cases cross-checked in pure X_eTaL |
| 3 | photo-lab | demos/photo-lab: filters and edges by rotation, SVD compression at several ranks, PNGs out |

## Saga 8 -- fetch and analyze (roadmap)

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | http | the http extension: bounded GET; tests against the web extension on loopback |
| 2 | digest | the digest extension (verifying fetched content) |
| 3 | fetch | demos/quakes: the USGS feed (opt-in `just live-quakes`), a saved copy for the golden |
| 4 | release-1 | catalog and pages reviewed, demos re-run, asks reviewed, retrospective here |

## Saga 9 -- native hook and macro helpers (blocked)

Blocked on asks E1 (a native hook in X_eTaL) and E2 (`.xtlm`). Until
then only the designs below are kept current.

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | native-survey | refresh the vendor; read what X_eTaL implemented (`[]S_VO` or a registration API); confirm A3-A6 against it |
| 2 | facades-native | each facade calls the native function directly; the bridge kept only as a fallback or retired; goldens unchanged (the proof that programs did not change) |
| 3 | bind-macro | `lib/Ffi.xtlm`: `ffi:b_ind<` generating a typed facade function from a signature; facades rewritten with it |
| 4 | release-2 | catalog, docs, retrospective |

Native hook design (for reference when E1 is discussed upstream;
research.txt): `[]S_VO` loads a library and resolves a symbol into an
ordinary callable whose X_eTaL type comes from the descriptor, e.g.

```
# Digest.xtl, once E1 lands
s := []S_VO "digest"                 # the package, found on XETAL_PATH
l:s_ha256 := { t -> "sha256" s_vo t } # or s gives typed functions by name
```

and today, through the bridge (A6):

```
# Digest.xtl, today
l:s_ha256 := { t -> t []N_PUT "ext:digest/sha256"; []N_GET "ext:digest/sha256" }
```

## Cross-cutting

- Refresh the vendored X_eTaL at a saga start or when an ask lands;
  never mid-step; its own commit; goldens and types re-run.
- When an ask lands, remove its workaround in the step that refreshes
  the vendor, and mark the ask landed.
- A function that belongs in X_eTaL itself (a clock is a fair
  candidate for a quad) is proposed as an ask, not kept here silently.
- Static linking of several extensions into one binary: each
  extension's `xetal_extension_v1` is unmangled, so two linked into one
  binary would collide; saga 3 step 1 puts it behind a `static`
  feature, used only outside the root workspace (feature unification
  would otherwise strip the symbol from the shared libraries).
- The browser: saga 3. Each later extension whose crate builds for
  `wasm32` gets a live page in the step that builds it.
