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

## 2026-10-05

- 00:08 `build` X_eTaL is no longer copied into this repository (the user, after X_eTaL's docs/vendoring.md): `XETAL_COMMIT` names the known-good commit (f823212); `just xetal` (`scripts/xetal.sh`) clones X_eTaL into `work/xetal/`, checks it out, builds and links `bin/xetal`; Cargo path dependencies, `xetal-x`'s `#[path]` modules, the build scripts, checks and recipes point at the clone; `check-vendor` checks the clone is at the commit and unedited; every script that builds runs `scripts/xetal.sh` first; `vendor/xetal/` (657 tracked files) removed. The gate passes unchanged (21 demos identical, every golden).

- 22:04 `fix` American spellings only (the user, after fixing X_eTaL-demos): `scripts/check-spelling.py` (copied from X_eTaL-demos; audio, video and fixture files skipped) and its self-test in the gate (`just check-spelling`); an audit found 55 British forms in 20 files (-our for -or, -re for -er, -yse for -yze, doubled l, and the like) and all were fixed, Rust identifiers included (scene's `Object::color`), the canvas listing golden with its doc string; CLAUDE.md rule 14.

- 20:56 `build` xetal-x follows the vendored CLI's new `main` (the terminal installed, `--cfg` flags, options before or after the subcommand: `command()` repeated verbatim) and its new crates (xetal-line, -tty, -system, -doc, -docsite, -doctest). Every reg-rs golden and Rust test passed unchanged; all 21 vendored demos identical under xetal-x. Asks E2 and E6 landed (vendored).

- 20:07 `vendor` X_eTaL f823212 vendored (from 7b70575: `.xtlm` macro libraries and the system macros, ask E2; options before the subcommand, ask E6; the terminal; the speed work).

## 2026-10-04

- 20:06 `docs` The triggered oscilloscope checked live: it ended by itself (9 s) and the user found it now looks like a scope.

- 19:39 `fix` The oscilloscope triggers (the user: "like a scope without triggering ... just moving wavy lines"): each frame draws from the left channel's first rising zero crossing after the playhead, both channels from it, so a steady chord stands still; frame-to-frame change measured at about 1,400 pixels before and 40 to 250 after; headless golden rebased and the recording re-made on purpose.

- 16:22 `docs` The oscilloscope replayed live: it ended by itself with the music (10 s), and the user confirmed the sound by ear.

- 15:42 `fix` The oscilloscope checked live with the user's go: the window drew both channels as the music played, but did not end with it -- the device's played count, converted back from the device's rate, stops just short of the last frame; it now ends within a tenth of a second of the end (stopped by hand within the agreed time).

- 15:18 `ext` The live oscilloscope (`just demo audio scope`): the synthesizer's music plays while a canvas window draws the waveform under the playhead (left above, right below), the bars streamed as the playhead nears them; `au:p_layed` (with no device a virtual clock); the instruments moved into an ordinary library beside the demos, `Instruments.xtl` (the synth's WAV unchanged; a first name `Synth.xtl` collided with `synth.xtl` on the case-insensitive file system); a headless reg-rs test pins frames and sound; recordings can take a demo's own sound (`wav=1`); demo lists skip capitalized (library) files.

- 14:58 `build` This repository's port is 8470 (the user's scheme: one port per X_eTaL repo, so their demos run side by side): `port` in the justfile, `just serve-pages` (bound to 127.0.0.1), trunk in the pages' docs, the web extension's planned default; CLAUDE.md rule 14.

- 14:56 `docs` Upstream swept: E2 (`.xtlm` macro libraries) and E6 (`--draw` before the subcommand) have landed in X_eTaL (not yet vendored); E1, E3, E4, E5 and the web-release are still open. docs/ideas.md: camera input and ONNX models, as notes only (the user's decision: no plan yet).

- 14:42 `ext` The array synthesizer: audio gains voices (`au:o_utput`, `au:q_ueue!`, `au:w_ait!`; a thread owns the device; with no device `XETAL_AUDIO_WAV` collects the sound in a WAV); `demos/synth.xtl` computes eight bars in X_eTaL (equal temperament, harmonics as a table of sines, envelopes, arpeggio, bass, stereo) and streams them; a Rust test reads a voice's WAV back; reg-rs pins the synth's WAV; recordings gain `NAME.sound` (spectrogram with the sound). A right-to-left slip (`t / 0.01 m_in 1.0`) that clipped the first render was caught by its RMS.

- 13:23 `docs` The visualizer checked live (with the user's go: the window, the spokes moving, the clean end); docs/parity.md updated (host, canvas, scene, audio and the visualizer done); Saga 7 retrospective.

- 13:01 `ext` The music visualizer (`extensions/audio/demos/spectrum.xtl`, `just demo audio spectrum`): the arpeggio plays while X_eTaL takes the window under the playhead, tapers it, finds the loudness at 8 octaves per channel by one inner product with cosine and sine tables, eases it, and lays out 16 mirrored spokes in the scene (bass blue, mid green, high orange); keys pause and seek. Recorded headlessly with its music (`videos.sh` muxes a frames spec's `audio=`); a headless reg-rs test pins three frames. The scene draws 2 px lines (cube golden and video rebased); scripted headless events are no longer capped at 256; `just demo` runs the release build; `videos.sh` safe for empty arrays on bash 3.

- 09:41 `ext` The audio extension (`extensions/audio`, alias `au:`; symphonia 0.5, cpal 0.15): open, info, chunk, play, pause, seek, position, window (the frames under the playhead), state, close; a device thread decodes ahead into the sound device and counts what it played; a virtual clock with no device (`XETAL_AUDIO=off`) for exact tests; generated Ogg/MP3/WAV tones and a 16 s Ogg arpeggio; 4 Rust and 5 reg-rs tests (a spectrum by inner product finds 440 Hz left, 880 Hz right).

## 2026-10-03

- 19:13 `ext` The scene extension (`extensions/scene`, alias `sc:`, linked into `xetal-x`): objects by id (polyline, segments, dots) with colors, an orbit camera with spin and mouse drag, perspective and anti-aliased lines drawn on the CPU (decision M7 revised from wgpu, so frames are the same headless), headless frames; `demos/cube.xtl`; 5 Rust tests, 4 reg-rs tests (the cube headless, frames pinned); `videos/cube`.

- 17:14 `plan` Launch step 5 (the six-repo snapshot tag) waits on X_eTaL's web-release, which picks the compatible versions; then this repo vendors that release and tags to match.

- 16:18 `docs` Ready for `.xtlm`: docs/ffi-macro.md designs the binding macro (`"s_hout : text -> text" ffi:b_ind< "hello/shout"`: channel kinds, exact expansions, the plan for the day it lands); docs/ffi/Hello.xtl is hello's facade expanded by those rules, by hand, and two reg-rs tests prove it (types equal to the facade's; the tour's golden through it).

- 15:40 `docs` Cross-repo audit: every ask (E1-E6) is in X_eTaL's queue (E6 Saga 30, E2 Saga 19, E4 Saga 13, E3 after Saga 21, E5 Saga 27, E1 Saga 23) and none has landed (84 commits past the vendored 7b70575), so the vendor stays; statuses updated. `check-vendor.sh` no longer pipes into `grep -q` under pipefail.

- 15:11 `fix` `xetal-x --version` first names itself and this repository's commit (research4's provenance blocker); `check-xetal-x.sh` checks it and no longer pipes into `grep -q`/`head` under pipefail (a SIGPIPE made it flaky).

- 13:59 `plan` research4 (../X_eTaL) adopted: the media saga is paused after step 2 (scene WIP on branch `wip/media-scene`; audio and the visualizer post-launch); Saga 6, launch: this repo's promotion blockers, the cross-repo audit, readiness for the `.xtlm` binding macro, the six-repo snapshot; Priorities and a promotion-blocker list in the plan; README labels the ABI and bridge experimental; later sagas renumbered 7-10.

- 13:54 `docs` The site shows recordings instead of live pages (the user's decision: extensions are native): `just videos` records every demo headlessly (vhs tapes typing `just demo EXT NAME --echo`; window demos from headless frames joined by ffmpeg), `just pages` builds a page of the recordings with the commands to run each; root `just demos` and `just demo EXT NAME`; README shows the notebook recording; the WebAssembly pages are no longer published (their code stays in `web/`). CLAUDE.md: the screen protocol.

- 12:46 `feat` canvas without a screen: `XETAL_HEADLESS=1` (no window; `XETAL_EVENTS` scripted, then close) and `XETAL_FRAMES=DIR` (every frame saved as PNG); the Life demo runs headless in the gate with its frames pinned.

- 12:16 `feat` The UI host and canvas: `crates/xetal-ext-ui` (a winit event loop on the main thread, created only when a program opens a window; jobs from the program's thread; event queues with frame ticks); `xetal-x` runs the program on a thread of its own and serves the main thread (all 20 vendored demos still identical); host-linked packages (`host = true`); the canvas extension (`cv:o_pen!`, `cv:s_how!`, `cv:n_ext!`, `cv:c_lose!`; softbuffer) with `demos/life.xtl` (Life in a native window); headless Rust and reg-rs tests, `just smoke` opt-in.

- 12:05 `plan` docs/parity.md: this repo against sw-MLPL's demo-extensions (boundary, extensions, demos). Saga 5, media (the user's request): the MP3 visualizer -- a UI host in `xetal-x` (the window on the main thread, the program pulling events and pushing scene patches), a retained 3D scene extension adapted from demo-extensions, an audio extension (Symphonia, CPAL), and the visualizer with X_eTaL doing the analysis; later sagas renumbered 6-9.

- 11:49 `plan` No SQLite in the browser (the user's decision): sqlite stays command-line only; browser storage or a client/server split (X_eTaL with extensions on a server, a WebAssembly X_eTaL UI in the browser over REST or WebSockets) noted as later demos, not extensions. Release 1 tagged `v0.1.0`.

- 11:26 `release` Release 1 (hello + clock + sqlite), research3's checklist: README "Start here" and value proposition with "libraries extend the vocabulary; macros extend the language; native extensions extend the machine"; `docs/status.md` generated by `scripts/status.py` (checked in the gate); the site's catalog lists every extension (live or command line); `just walkthrough` (a fresh clone built and release 1 run against the goldens) passes; stale-ask sweep (E1 filed as X_eTaL Saga 23, E2 Saga 19, none landed: vendor kept); saga 4 retrospective.

- 10:39 `docs` SQLite in the browser probed: rusqlite 0.40 `bundled` reaches wasm32 through `sqlite-wasm-rs`, whose build needs a clang that targets WebAssembly (Apple's does not; LLVM's would). Recorded on sqlite's page and in the plan; sqlite stays command-line only until LLVM is installed (the user's choice).

- 09:53 `ext` The data notebook (`extensions/sqlite/demos/notebook.xtl`): Mauna Loa CO2 1959-2025 (NOAA GML and Scripps, provenance bundled) imported into SQLite; SQL for the overview and decade means, X_eTaL (with `Stats`) for the yearly rise, a least-squares line and its residuals (the rise is speeding up), a histogram as an APL bar chart and the curve, both SVG; checked in Chrome; reg-rs pins output and pictures. Demo recipes load every extension and draw into `work/draw`. Ask E6 (`xetal --draw DIR run FILE` misparses).

- 09:02 `ext` sqlite CSV import: `db sq:i_mport "path.csv"` or `"table=path.csv"` (an RFC 4180 reader; INTEGER, REAL or TEXT inferred per column; empty fields NULL; one transaction; existing table or ragged records refused; confined paths); Rust tests of the reader, types and imports; reg-rs `sqlite-import` (planets.csv: Kepler's third law from SQL and X_eTaL); list and types goldens rebased for the new function.

- 08:52 `plan` research3.txt (../X_eTaL) adopted for this repo: release 1 is hello + clock + sqlite with the data notebook ("Do SQLite"); web, image, linalg, http and digest are roadmap (README and plan); saga 4 gains sqlite-live and release-1 (status matrix, stale-ask sweep, full catalog, fresh-user walkthrough, the three-ways phrase, a release tag proposed).

- 08:42 `ext` The sqlite extension (`extensions/sqlite`, alias `sq:`; rusqlite with bundled SQLite): `db sq:e_xec sql` (rows changed), `sq:n_ums` (a Float matrix, NULL as NaN), `sq:t_exts` (cells as a Char matrix), `sq:c_ols`, `sq:q_uote`; databases named by confined relative paths (or `XETAL_SQLITE_ROOT`) or `:memory:`; 4 Rust tests through the loader, 6 reg-rs tests; ask E5 (errors name the calling line).

- 08:29 `plan` Saga 3 (live) retrospective; saga archived.

- 08:26 `feat` clock live: clock reads the browser's clocks on wasm32 (`Date.now`, `performance.now`; std's panic there); `extensions/clock/web/` runs the bridge-cost demo in the browser with clock and hello both linked statically (the `static` feature with two extensions); native test against the masked golden; pages rebuilt and checked in Chrome (about 16,000 calls/s and 1.7 M Floats/s in the browser).

- 07:16 `build` The live site: `scripts/build-pages.sh` (`just pages`: every `extensions/NAME/web` built with trunk into `pages/NAME/` under `/X_eTaL-extensions/`), `scripts/build-catalog.py` (`pages/index.html`, a card per live page), `just serve-pages`, `.github/workflows/pages.yml` (uploads the committed `pages/`, as X_eTaL-demos); GitHub Pages enabled (workflow); README link. Checked at the real base path in Chrome.

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
