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

## 2026-10-02

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
