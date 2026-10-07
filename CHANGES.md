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

## 2026-10-07

- 13:00 `ext` Voxels 1, `just demo scene voxels-chunk`: a 16 by 16 by 16 chunk as an array, built from a height field in X_eTaL (grass, three of dirt, stone, water below 7, sand at the shore, a tree on the hill), counted, printed as character slices, and seen from above (each column's top by a max-reduction, the block there by comparing heights); two pictures. The library `demos/Voxels.xtl` (`vx:`) begins. Golden and terminal recording.

- 12:10 `plan` Saga 13, voxels (voxels-chunk, voxels-faces, scene-faces, voxels-solid, voxels-world, voxels-release), and Saga 14, the voxel game, outlined (the user: finish photo lab, then the voxel sagas; each demo shown before the next). The photo saga archived.

- 11:50 `release` Photo lab saga done: the photo lab recorded as its pictures in turn (a new recording kind, `videos/NAME.pics`: the demo runs, its PNGs are the frames); the image page shows it; README, CLAUDE.md rule 12, status; Saga 10 retrospective.

- 11:20 `ext` The photo lab, `just demo image photo-lab`: Buzz Aldrin on the Moon (NASA AS11-40-5903, public domain, fetched once with the http extension; `demos/data/PROVENANCE.txt`) as a 240 by 246 gray array; blur, sharpen and Sobel edges by rotation (the Life idiom), sepia by an inner product with a 3 by 3 matrix, the SVD rebuilding it from 5, 20 and 50 singular values (94%, 98%, 99% of the energy; 4%, 17%, 41% of the numbers; RMS error 0.107, 0.066, 0.039); three PNG sheets in `work/photo-lab/`; under 3 s. Golden `image-demo-photo-lab`.

- 10:30 `ext` The linalg extension (`extensions/linalg`, `"la:" u_se< "Linalg"`, nalgebra 0.33): `la:s_olve` (a vector or columns on the right; singular refused), `la:i_nverse`, `la:d_et`, `la:l_stsq` (by the SVD), `la:e_ig` and `la:e_igVecs` (symmetric, ascending), `la:s_vdS`, `la:s_vdU`, `la:s_vdV` (thin, descending). Rust tests with known answers; reg-rs: list, types, the error for a singular matrix, and `linalg-cross-check`: seven small cases recomputed in pure X_eTaL (Cramer's rule, the 2 by 2 eigenvalue formula, inverse times the matrix, U diag(s) V').

- 09:30 `ext` The image extension (`extensions/image`, `"im:" u_se< "Image"`): PNG and JPEG as Float arrays from 0 to 1 (gray h by w, color h by w by 3; each 8-bit level k as k / 255, so pictures round-trip exactly), `im:g_ray` (luminance), `im:s_ize`, `im:w_rite!` (PNG or JPEG by the name, clamped), `im:r_esize` (Lanczos); confined paths (`XETAL_IMAGE_ROOT`); the `image` crate with PNG and JPEG only. Rust tests on generated pictures; reg-rs: list, types, a round trip from X_eTaL, the error for a path outside.

- 08:30 `docs` docs/voxels.md (the user: analyze ../avoxelgame-fork for a voxel mini game, built up by `voxels-*` demos): what the Dyalog APL game does and what carries over; this repo has no SDL (winit and softbuffer on the CPU) and needs filled depth-tested faces, a first-person camera and key-up/mouse-motion events in scene; measured: the exposed-face mask costs 1.75 ms at 16 by 16 by 16 and 13 ms at 16 by 128 by 16 in X_eTaL, the bridge 0.35 microseconds a number (a 256 by 256 color frame 71 ms), so faces cross per chunk change and stay in Rust; 16-cubes recommended (sky light by a column scan, block light and water reachable); eight demos, two needing nothing new; the mini game Gem Hunt. Analysis only, not scheduled.

- 00:10 `plan` Saga 10, photo lab, started after the fetch saga (the user's order): image, linalg, photo-lab, photo-release. The fetch saga archived.

## 2026-10-06

- 09:20 `release` Fetch saga done: the earthquake report recorded at the command line (`videos/quakes.tape`; the live feed never recorded); the http page shows it; parity (http-client done); ask E8 (a library named like its program, on a case-insensitive file system, reports an import cycle); status; Saga 11 retrospective.

- 08:55 `test` http's Rust tests take turns (the last response is the extension's state, and two tests in one binary raced on it) and no longer assume which runs first: the gate caught it, ten runs in a row pass.

- 08:40 `ext` The earthquake report, `just demo http quakes`: the USGS M2.5+ week feed saved 2026-10-06 with the http extension (286 quakes; public domain; `demos/data/PROVENANCE.txt`), checked against its SHA-256 (digest), into SQLite; SQL gives the largest three and quakes per day, X_eTaL the half-unit histogram, the log counts and the Gutenberg-Richter b-value by Aki's maximum likelihood above the feed's completeness of 4.5 (1.16 from 114 quakes; the same in Python); a world map (quakes counted in 10-degree cells), the histogram and the log-count line as SVG. The report is the library `demos/Seismic.xtl` (not `Quakes.xtl`: case-insensitive file systems), shared with `quakes-live.xtl` (`just live-quakes`, the live feed; run once here: 285 quakes). Golden `http-demo-quakes`.

## 2026-10-05

- 07:55 `ext` The digest extension (`extensions/digest`, `"dg:" u_se< "Digest"`): SHA-256 (64 hex digits) and CRC-32 (IEEE) of text and of confined files read in 64 KiB pieces (`sha2`, `crc32fast`). Rust tests: the standard vectors (empty, `abc`, `123456789`, a non-ASCII letter, checked against Python's hashlib and zlib), a file bigger than one piece, confinement; reg-rs: list, types, the vectors from X_eTaL and a file against its recorded SHA-256, the error for a path outside.

- 07:30 `ext` The http extension (`extensions/http`, `"ht:" u_se< "Http"`): bounded GET through ureq 3 (rustls), only http and https; limits on size (`XETAL_HTTP_MAX`, 16 MiB), time (`XETAL_HTTP_TIMEOUT`, 30 s) and redirects (5); a status other than 2xx is an error naming it; `ht:s_tatus` and `ht:h_eader` of the last response; `ht:s_ave!` writes a confined file whole or not at all. Rust tests fetch from the web extension's server on loopback (text, headers, a redirect and a loop, 404, a download, the size and time limits); reg-rs: list, types, two errors, and `http-fetch`, an X_eTaL program against `python3 -m http.server` on loopback.

- 06:40 `plan` Saga 11, fetch and analyze, started before photo lab (the user: "earthquake feed first, then photo lab"): http, digest, quakes, fetch-release. The web saga archived. The user saw both web demos live (54 s with their go): "looked good".

- 06:10 `release` Web saga done. Recordings of the live page and TodoMVC as a browser shows them: a new recording kind, `videos/NAME.web` (`scripts/videos.sh`): the demo served on a free port, driven by the spec's steps (`shot PATH`, `post PATH ...`), each shot rendered by headless Chrome with a temporary profile (stopped once the file is written; no window). The web page shows them; parity (http-server and TodoMVC done), README, CLAUDE.md rule 12, status; Saga 9 retrospective in docs/plan.md.

- 05:20 `ext` TodoMVC on the web, `just demo web todomvc` (http://127.0.0.1:8470/): HTML forms, the todos kept in SQLite (`work/todos.db`); the X_eTaL program routes each request, turns the filter into SQL, counts what is left from the `done` column and writes the page, SQL writes (and escapes) each item; changes redirect back with 303. New `wb:h_eader! "Name: value"` (checked; content type and length refused), with Rust tests of a redirect. `web-demo-todomvc` drives it with `curl`: add (an HTML-looking title escaped, an empty one ignored), toggle, filters, delete, the list kept across a restart, clear completed. Goldens rebased on purpose: the new function in the listing and types, two error lines three lines lower in the facade.

- 04:30 `ext` The live page, `just demo web live` (http://127.0.0.1:8470/): Conway's Life kept by the X_eTaL program, one generation per request, drawn as an SVG path built in X_eTaL (`m_ap` over the live cells, joined); New board, Glider and Quit are requests too. `XETAL_WEB_PORT` replaces the port a program asks for, so tests run a demo on any free port; `web-demo-live` drives it with `curl` (a glider stepped three times, exact SVG).

- 03:45 `release` Tagged v0.2.0 at 9dec588 (the user's yes): an annotated tag only, its notes saying it is built against X_eTaL v0.1.0 (512b3ee) and listing what came since v0.1.0; no GitHub Release object (left for the launch, as in X_eTaL).

- 03:25 `build` X_eTaL v0.1.0 (512b3ee, from f823212) is the known-good commit (the user relayed X_eTaL's release; pin for every sibling). `xetal-x` follows the CLI's renamed crate (`xetal-docsite` to `xetal-docsearch`). Every golden and all 22 demos under `xetal-x` pass unchanged but one: macros are now hygienic (MC30), so `xetal expand` shows `g1:a` for a name the binding macro binds; `hello-ffi-macro` compares with those prefixes read away. Asks swept: E4 partly landed (`[]TS`, `[]D_L`, no monotonic clock); E1, E3, E5, E7 still open.

- 03:10 `ext` The web extension (`extensions/web`, `"wb:" u_se< "Web"`): axum 0.8 on a tokio runtime of its own on a background thread, 127.0.0.1 only, a queue of 64 requests (503 beyond), replies within 10 s (`XETAL_WEB_TIMEOUT`; 504 after); the program drives: `wb:s_erve!`, `wb:n_ext!` ("GET /path" or "none"), the request's parts and decoded query/form fields, `wb:c_ontent!`, `wb:r_eply!`, `wb:f_iles!` (a directory served directly), `wb:s_top!`. Its facade is eleven `ffi:b_ind<` lines. Rust tests with a real HTTP client on loopback (the whole flow, 503, 504, files, fields); reg-rs: list, types, three errors, and `web-serve`, an X_eTaL program answering four `curl` requests. The extension template now writes its facade with the binding macro. Step inserted to pin X_eTaL v0.1.0 (512b3ee), relayed by the user.

- 01:55 `build` The gate fails when the site is out of date: `scripts/check-pages.sh` (`just check-pages`) builds it into `work/pages-check` (`PAGES_DIR`) and compares it with `pages/`, ignoring only the commit it was built at; a stale page was shown to fail. Found when the published site still said X_eTaL is vendored.

- 01:40 `plan` Saga 9, web, started (the launch was postponed; the user: proceed): pages-fresh, web, live-page, todomvc, web-release. The macros saga archived.

- 01:30 `docs` Site rebuilt: its build instructions said X_eTaL is vendored; now `just xetal` first (stale since the devendor step).

- 01:15 `release` Macros saga done: docs/ffi-macro.md describes the facades as built (all six written with `ffi:b_ind<`; clock's `t_ime` by hand; errors at the signature line); README says where libraries, macros and native extensions meet (every facade written with a macro); status says `.xtlm` is in use; asks E5 and the 2026-10-05 note updated; the sqlite and hello pages follow; Saga 8 retrospective in docs/plan.md.

- 01:00 `ext` Every facade written with the binding macro: hello, clock, sqlite, canvas, scene and audio are now one `ffi:b_ind<` signature line per function (clock's `t_ime` still by hand) and `l:package`. Every function's type is unchanged (the types goldens gain only `l:package : Char`), every demo golden is unchanged; the eight error goldens now locate the error at the function's signature line. The `xetal-play` bridge test and the shell's store serve `Ffi.xtlm`.

- 00:28 `feat` The binding macro, `lib/Ffi.xtlm` (`"s_hout : text -> text" ffi:b_ind< "hello/shout"`): about 60 lines of X_eTaL string work; `xetal-x` puts the repo's `lib/` on `XETAL_PATH`. Its expansion of hello's seven signatures equals `docs/ffi/Hello.xtl` exactly, its types the facade's, and the tour through it its golden (three reg-rs tests). Ask E7: `xetal type` decides library-or-program before expanding macros (workaround: `l:package`). docs/ffi-macro.md describes it as built.

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
