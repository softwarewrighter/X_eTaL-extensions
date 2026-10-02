# foundation

Saga 1 of X_eTaL-extensions (docs/plan.md): the process, the vendored
interpreter, and the native extension boundary -- ABI V1, the SDK, a
hello extension built as a real .dylib/.so, and the loader that
validates and calls it.

Model: ../X_eTaL-demos, ../X_eTaL-games, ../X_eTaL-libraries (same
vendoring scripts, gate and process), ../X_eTaL (CHANGES.md), and
../../sw-ml-study/demo-extensions (the ABI/SDK/loader design for
sw-MLPL: copy and adapt, never depend on it).

Rules: extensions are Rust cdylibs behind .xtl facades; X_eTaL only
through the vendored snapshot in vendor/xetal/; missing features and
bugs go in docs/xetal-asks.md, workarounds named; .xtlm macro helpers
wait until X_eTaL supports them. Every step: `just gate` passes, docs
(README, CHANGES.md, plan, asks, pages) updated, .gitignore sane, a
detailed commit to main including .agentrail/, `agentrail complete`,
push.

## Steps

1. scaffold -- process, CLAUDE.md/AGENTS.md, README, COPYRIGHT,
   LICENSE, CHANGES.md, justfile, gate, docs/plan.md,
   docs/xetal-asks.md, docs/research.txt.
2. vendor-xetal -- `just vendor [REF]`, vendor/xetal/VENDORED,
   `just xetal`, `just eval`, gate check.
3. abi -- root Cargo workspace; crates/xetal-ext-abi (descriptor,
   values, errors, validation) with tests; cargo test/fmt/clippy in
   the gate; docs/abi-v1.md.
4. sdk-hello -- crates/xetal-ext-sdk (xetal_extension!, panic-safe
   trampolines, free callback); extensions/hello cdylib + manifest.
5. loader -- crates/xetal-ext-loader: manifests, libloading,
   validation, registry; hello loaded dynamically and statically.
