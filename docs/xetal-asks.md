# Asks for X_eTaL

Features the extensions need that X_eTaL does not have yet, and bugs
the extensions uncovered. This repo does not change X_eTaL: each ask
is filed here (and taken to `../X_eTaL`), the extension uses the
workaround noted below or waits, and the workaround is removed when
the ask lands in a vendored release (`vendor/xetal/VENDORED`).

Each entry: status (open, filed, landed, dropped), kind (feature, bug
or speed), which extensions need it, why, a minimal repro or example,
and the workaround in use.

| # | Status | Kind | Ask | Extensions | Workaround |
| - | ------ | ---- | --- | ---------- | ---------- |
| E1 | open | feature | a native hook: a way for X_eTaL code to call a function in a native library (`[]S_VO`, reserved in lang-choices QD4 / section 15), with the function's X_eTaL type from the library's descriptor | all | the bridge host `xetal-x` (plan A6): a store routing `ext:` paths of `[]N_PUT` / `[]N_GET` to native functions, text only |
| E2 | open | feature | `.xtlm` macro libraries (MC10-MC13, X_eTaL Saga 19); same as `../X_eTaL-libraries` ask X1 | macro helpers (`ffi:b_ind<`, saga 7) | none: the helpers wait |
| E3 | open | feature | the CLI as a library: a host entry point (`xetal_cli::run_with(args, store)` or similar) so a host can be `xetal` with its own store | the bridge host `xetal-x` | `xetal-x` compiles the vendored CLI's source files as `#[path]` modules and repeats its 60-line `main` with a different store; breaks if the CLI's modules are reorganized |
| E4 | open | feature | the time: a quad giving the current time (APL's quad TS) and a monotonic or CPU clock (quad AI) | clock (and anything timing itself) | the clock extension, through the bridge |

Asks already filed by the sibling repos
(`../X_eTaL-demos/docs/xetal-asks.md`,
`../X_eTaL-games/docs/xetal-asks.md`,
`../X_eTaL-libraries/docs/xetal-asks.md`) that an extension also hits
are copied here with the extension named, so this list stands on its
own.

## Details

### E1: a native hook

X_eTaL has no way to call native code. lang-choices reserves `[]S_VO`
for "channels to special facilities (graphics, a Rust dynamic
library)" as the FFI-like escape hatch, and the X_eTaL plan schedules
it after the quads ("Later -- system I/O"), names to be decided with
the user.

What the extensions need, smallest first:

1. A host can register named functions with X_eTaL types before a
   program runs (a registration API in the evaluator and type
   checker, as the store is registered with
   `xetal_store::install`). This alone lets a host like `xetal-x`
   give programs real typed functions instead of a text channel.
2. A program can load a native package itself: `[]S_VO "digest"`
   finds `digest`'s manifest on `XETAL_PATH`, loads the library for
   this platform, and gives its functions, typed from the descriptor
   (this repo's ABI V1, `docs/abi-v1.md`, offered as a starting
   point).

Example of what should work:

```
"dg:" u_se< "Digest"
dg:s_ha256 "abc"
```

with `Digest.xtl` binding `s_ha256` to the native function, of type
`String -> String`.

Today: there is no such name; `[]S_VO` is an unknown name.

Workaround: the bridge host (plan A6). `xetal-x` installs a store in
which `t []N_PUT "ext:digest/sha256"` calls the native function on
the text `t` and `[]N_GET "ext:digest/sha256"` gives its reply. Only
text crosses (numbers as `f_ormat` / `n_umbers` text); each call costs
two system calls and a formatting round trip; and a program must be
run with `xetal-x run`, not `xetal run`. Every facade hides the
channel behind ordinary typed functions, so programs do not change
when E1 lands.

### E2: `.xtlm` macro libraries

Decided upstream as MC10-MC13 (lang-choices), planned as X_eTaL Saga
19; asked for by `../X_eTaL-libraries` (X1, X2). Here it would let a
macro library generate facade functions from signatures:

```
# Ffi.xtlm (sketch)
m:b_ind< := { sig target -> ... }    # "String -> String" ffi:b_ind< "digest/sha256"
```

Workaround: none. Facades are written by hand until it lands.

### E3: the CLI as a library

`xetal-x` must behave exactly as `xetal` (every subcommand, flag and
message) and differ only in its store. The CLI is a binary crate with
private modules, so `xetal-x` compiles the vendored files
(`components/cli/crates/xetal-cli/src/{args,context,echo,live,once,stages}.rs`)
as `#[path]` modules, unchanged, and repeats `main.rs` with
`ExtStore` wrapping the `Drawing` store. `scripts/check-xetal-x.sh`
checks that every vendored demo gives the same output under both.

What would remove the workaround: a library target in `xetal-cli`
exposing the CLI's run with a caller-supplied store, e.g.

```
pub fn main_with(args: impl IntoIterator<Item = OsString>,
                 store: impl FnOnce(&Command) -> Arc<dyn Store>) -> ExitCode
```

Not urgent: E1 (a native hook) would make the bridge itself
unnecessary.

### E4: the time

X_eTaL has no way to read a clock, so a program can neither stamp what
it writes nor time what it does. APL has the timestamp (quad TS: year,
month, day, hour, minute, second, millisecond) and the accounting
information (quad AI: CPU and connect time). A quad pair such as
`[]T_S @` (the timestamp as a 7-element Int vector, UTC or local) and
`[]M_S @` (monotonic milliseconds, Float) would do. The clock extension
(`extensions/clock`, `ck:n_ow`, `ck:i_so`, `ck:m_illis`, `ck:t_ime`)
is the workaround; it needs `xetal-x`. Proposed here rather than kept
silently as an extension (plan, cross-cutting).
