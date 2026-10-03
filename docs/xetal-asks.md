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
| E1 | filed (X_eTaL plan Saga 23, host bindings and native packages, which may start from this repo's ABI V1) | feature | a native hook: a way for X_eTaL code to call a function in a native library (`[]S_VO`, reserved in lang-choices QD4 / section 15), with the function's X_eTaL type from the library's descriptor | all | the bridge host `xetal-x` (plan A6): a store routing `ext:` paths of `[]N_PUT` / `[]N_GET` to native functions, text only |
| E2 | filed (X_eTaL Saga 19, in progress: MC10-MC13 decided, not yet implemented in the vendored 7b70575 nor upstream HEAD on 2026-10-03) | feature | `.xtlm` macro libraries (MC10-MC13, X_eTaL Saga 19); same as `../X_eTaL-libraries` ask X1 | macro helpers (`ffi:b_ind<`, saga 9) | none: the helpers wait |
| E3 | filed (X_eTaL queue item 6, after Saga 21) | feature | the CLI as a library: a host entry point (`xetal_cli::run_with(args, store)` or similar) so a host can be `xetal` with its own store | the bridge host `xetal-x` | `xetal-x` compiles the vendored CLI's source files as `#[path]` modules and repeats its 60-line `main` with a different store; breaks if the CLI's modules are reorganized |
| E4 | filed (X_eTaL queue item 5, Saga 13 steps: `[]TS`, `[]D_L`) | feature | the time: a quad giving the current time (APL's quad TS) and a monotonic or CPU clock (quad AI) | clock (and anything timing itself) | the clock extension, through the bridge |
| E5 | filed (X_eTaL queue item 9, Saga 27, readable type errors) | feature | an error raised inside a library function also names the program line that called it (a call trace) | every facade (errors point into the facade's helper, not the caller) | the extension's message names the function (`ext:sqlite/nums: ...`) |
| E6 | filed (X_eTaL queue item 2, Saga 30, right after its guard) | bug | `xetal --draw DIR run FILE` fails (clap takes `run` as the SCRIPT argument and FILE as a subcommand); `xetal run --draw DIR FILE` works | the sqlite notebook's test and every demo recipe | `--draw` written after `run` |

Swept 2026-10-03 (twice) against upstream X_eTaL (84 commits past the
vendored 7b70575): none of these has landed, so the vendor is not
refreshed. Every one is now in X_eTaL's queue (docs/plan.md there,
"The sibling repositories' asks come first"): E6 in Saga 30 (item 2),
E2 in Saga 19 (item 3), E4 in Saga 13's steps (item 5), E3 after Saga
21 (item 6), E5 in Saga 27 (item 9), E1 in Saga 23. The user took E3
to E6 there.

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
# Hello.xtl, once .xtlm lands
"ffi:" u_se< "Ffi"
"s_hout : text -> text" ffi:b_ind< "hello/shout"
```

The design, the exact expansions and their tests: docs/ffi-macro.md.

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

### E5: errors name the calling line

An error raised inside a library function is reported at the
library's line, not the program's. Every facade calls native code from
a private helper, so every native error points into the facade:

```
"sq:" u_se< "Sqlite"
":memory:" sq:n_ums "selec 1"
```

```
error[io]: []N_GET: ext:sqlite/nums: near "selec": syntax error in selec 1 at offset 0 at ./lib/Sqlite.xtl:12:17
```

Wanted: the program's line as well (`called from main.xtl:2:1`), as a
short trace of library calls. Workaround: each native error message
names its function, so the user can find the call.

### E6: `--draw` before the subcommand

```
xetal --draw out run prog.xtl
error: the subcommand 'prog.xtl' cannot be used with:
  --draw <DIR>
```

`xetal --help` lists `--draw <DIR>` as an option of `xetal` itself, so
it reads as if it may come first; clap then takes `run` as the
optional SCRIPT positional. `xetal run --draw out prog.xtl` works, as
does `XETAL_DRAW=out xetal run prog.xtl`. Workaround: `--draw` after
`run` (the demo recipes and the notebook's reg-rs test).
