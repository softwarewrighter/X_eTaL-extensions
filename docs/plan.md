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
| A2 | **One Cargo workspace** at the root for this repo's crates (`crates/*`, `extensions/*`), separate from the vendored X_eTaL's own workspaces; everything builds into `./target` (`.cargo/config.toml`). | One `cargo test` covers the ABI, SDK, loader and every extension. |
| A3 | **ABI V1** (`crates/xetal-ext-abi`, `docs/abi-v1.md`): every extension exports one symbol, `xetal_extension_v1`, returning a `#[repr(C)]` descriptor: size and ABI version first, name and version (UTF-8 slices), a bounded table of functions (name, arity 0 to 2, the X_eTaL type signature such as `Char -> Char`, a one-line doc, the trampoline), a reserved field that must be zero. Values carry a fixed-width tag and a payload: X_eTaL's scalars (Bool, Int as i64, Float as f64), text (UTF-8, a Char vector) and dense row-major arrays of Bool, Int, Float or Char of rank 0 to 9; an error is a code and a message. Bounded: 64 Mi elements per value, 16 KiB of descriptor text, 1,024 functions. | demo-extensions' ABI V1, reduced to X_eTaL's types; the size/version prefix and zero reserved fields reject layout drift; the signature lets a future native hook (E1) type the function without a facade. |
| A4 | **Safety at the boundary.** Validation is the only unsafe pointer-reading code; it copies everything into owned Rust values before registration. Rust panics inside an extension are caught in the extension (SDK trampolines) and returned as an error status, never unwound across `extern "C"`. A result lives in storage the extension keeps until its next call on the same thread; the host copies it at once. | Unwinding across C is undefined; a loader that keeps foreign pointers is a use-after-free waiting to happen. |
| A5 | **SDK and loader.** `crates/xetal-ext-sdk`: what an author writes (a `xetal_extension!` macro over plain Rust functions of `&[Value] -> Result<Value, String>`), generating the descriptor and the panic-safe trampolines, keeping each result until the next call. `crates/xetal-ext-loader`: resolves a package, `libloading`s it, validates, and registers its functions under the extension's namespace, checking arity before every call, keeping the `Library` alive as long as any function. The same descriptor can also be linked statically (tests, the browser later). | The author never writes `unsafe`; the host never trusts the extension. |
| A6 | **A bridge host until X_eTaL has a native hook.** `crates/xetal-x`: the vendored X_eTaL interpreter plus the loader, installing a store that routes paths starting `ext:` to native functions and everything else to the disk: `request []N_PUT "ext:digest/sha256"` calls the function on the request text and keeps the reply; `[]N_GET "ext:digest/sha256"` takes it. It is a documented workaround (ask E1), text-only (numbers cross as `f_ormat` / `n_umbers` text), and every facade hides it behind ordinary functions so programs never see it. Programs run with `xetal-x run FILE` instead of `xetal run FILE`. | Lets every extension be used end to end from X_eTaL today with no change to X_eTaL; it is APL's shared-variable channel, the model `[]S_VO` names. Replaced when E1 lands. |
| A7 | **An extension is a sub-project**, `extensions/<name>/`: `Cargo.toml` (a `cdylib` + `rlib`), `src/lib.rs`, `extension.toml` (name, version, ABI, namespace, the facade file, one native artifact per target triple), `<Name>.xtl` (the public facade), `tests/` (Rust tests through the loader, and `*.xtl` programs with `expected/` goldens run through the bridge), and a page `docs/ext/<name>.md`. | demo-extensions' package layout; each extension stands alone. |
| A8 | **Facade conventions** follow X_eTaL's style guide (lang-choices section 16) and `../X_eTaL-libraries` A6: `UpperCamel.xtl`, exports under `l:`, private helpers unprefixed, predicates `?`, effects `!`, a header with the import line and recommended alias, no export shadowing a built-in, no name shadowing a standard library. Exports are typed as if native (ordinary X_eTaL types), so the facade's types are pinned (`xetal type`) and survive the switch from the bridge to E1. | The facade is the extension's interface; its types are the contract. |
| A9 | **Pure fallbacks where cheap.** When an extension's function has a reasonable pure X_eTaL definition (CRC-32, a small determinant), the facade's tests compare native and pure results; the pure version is not exported. Where there is none (the clock, regular expressions), there is no fallback. | Golden behavior is cross-checked, not only self-consistent. |
| A10 | **Macro helpers wait for X_eTaL** (ask E2, X_eTaL MC10-MC13): a `.xtlm` that turns a signature into a facade function (`"f64 f64" ffi:b_ind< "linalg/det"`). Designed on paper only (saga 4), never emulated. | As in `../X_eTaL-libraries` A9. |
| A11 | A missing X_eTaL feature or bug an extension uncovers is **not** fixed here nor hidden: it goes in `docs/xetal-asks.md` (status, kind, extensions, why, minimal repro, workaround) and on the extension's page. | X_eTaL owns its language decisions; this repo is a consumer. |
| A12 | **Names, not homes**: an extension is identified by its name (`digest`), never a GitHub coordinate (the repos may move to `sw-array-languages`, research.txt). `just` is the entry point (recipes call `scripts/*.sh`); `CHANGES.md` gets a line for every commit; docs are ASCII-only markdown (`sw-markdown-checker`). | Same process as the sibling repos. |
| A13 | Dependencies are few, well known and permissively licensed (`libloading`, and per extension e.g. `sha2`, `regex`, `png`); each extension's page lists its crates. Nothing is downloaded at run time. | Small, auditable extensions. |
| A14 | **Copied from demo-extensions, never dependent on it.** Design, docs and code from `../../sw-ml-study/demo-extensions` (same author, MIT) may be copied and adapted to X_eTaL's types, with the source file noted in a comment; no Cargo path, git or build dependency on that repo or on sw-MLPL. | Reuse proven work while this repo builds and moves on its own. |

## Layout

```
crates/
  xetal-ext-abi/         the C ABI V1: layout, values, validation
  xetal-ext-sdk/         what extension authors use (xetal_extension!)
  xetal-ext-loader/      packages, dynamic loading, registry
  xetal-x/               the bridge host: vendored xetal + loader + ext: store
extensions/<name>/       one sub-project per extension
  Cargo.toml src/lib.rs  the cdylib
  extension.toml         the package manifest
  <Name>.xtl             the public facade
  tests/                 Rust tests; *.xtl programs and expected/ goldens
docs/ext/<name>.md       the extension's page
docs/plan.md             this plan
docs/xetal-asks.md       what the extensions need from X_eTaL
vendor/xetal/            the vendored X_eTaL (never edited)
scripts/                 the logic behind the just recipes
```

## The catalog

Ranked by what they prove first, then by usefulness. Aliases are
recommendations (the importer chooses, MC3), distinct from the
standard libraries' (`c:`, `m:`, `s:`) and `../X_eTaL-libraries`'.

| Extension | Facade, alias | What | Native crates | Pure fallback | Saga |
| --------- | ------------- | ---- | ------------- | ------------- | ---- |
| hello | `Hello`, `hx:` | the smallest proof: an answer, add two numbers, echo text, a typed error, a contained panic, the sum of a Float array | none | yes (trivial) | 1-2 |
| clock | `Clock`, `ck:` | wall-clock time (Unix seconds, ISO 8601 text), a monotonic millisecond counter, elapsed time of a block for benchmarks | std | none (X_eTaL has no clock) | 2 |
| digest | `Digest`, `dg:` | SHA-256 and CRC-32 of text, hex | `sha2`, `crc32fast` | CRC-32 | 3 |
| regex | `Regex`, `rx:` | match?, find (first, all), capture groups, replace, split by pattern | `regex` | none | 3 |
| linalg | `Linalg`, `la:` | determinant, inverse, solve, least squares, eigenvalues of a symmetric matrix, on Float matrices | `nalgebra` | small determinant and solve (Gauss-Jordan) | 3 |
| png | `Png`, `pn:` | write a matrix as a grayscale PNG, an n by m by 3 array as color; read one back as numbers | `png` | none | 3 |
| sqlite (later) | `Sqlite`, `sq:` | open, parameterized query to a matrix of text | `rusqlite` | none | later |
| http (later) | `Http`, `ht:` | bounded GET of text | `ureq` | none | later |

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
| 2 | bridge | the `ext:` store (A6): request / reply per function, errors as X_eTaL errors, `xetal-x run --ext DIR`; `just run-x FILE` |
| 3 | hello-facade | `Hello.xtl`, its `*.xtl` tests and goldens through the bridge, pinned facade types, docs/ext/hello.md, the facade test runner (`just test-ext NAME`) in the gate |
| 4 | clock | the clock extension end to end (a timing demo: `'+ r_/ r_ange` at growing sizes) |

## Saga 3 -- sample extensions

| # | Step slug | Delivers |
| - | --------- | -------- |
| 1 | digest | digest, with the pure CRC-32 cross-check |
| 2 | regex | regex |
| 3 | linalg | linalg, numbers as text through the bridge; its cost measured |
| 4 | png | png (writes images the sibling demos could use) |
| 5 | release-1 | catalog and pages reviewed, examples re-run, asks reviewed, retrospective here |

## Saga 4 -- native hook and macro helpers (blocked)

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
  binary would collide. `__xetal_extension::descriptor()` (mangled) is
  the static path; when a host links more than one, the unmangled entry
  moves behind a cargo feature. Not needed while the loader is dynamic.
- The browser: extensions are native; the web build would need the
  statically linked form (A5) compiled to WebAssembly. Out of scope
  until a demo asks for it.
