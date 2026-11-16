# Asks for X_eTaL

Features the extensions need that X_eTaL does not have yet, and bugs
the extensions uncovered. This repo does not change X_eTaL: each ask
is filed here (and taken to `../X_eTaL`), the extension uses the
workaround noted below or waits, and the workaround is removed when
the ask lands in the X_eTaL commit this repository builds on (`XETAL_COMMIT`).

Each entry: status (open, filed, landed, dropped), kind (feature, bug
or speed), which extensions need it, why, a minimal repro or example,
and the workaround in use.

| # | Status | Kind | Ask | Extensions | Workaround |
| - | ------ | ---- | --- | ---------- | ---------- |
| E1 | filed (X_eTaL plan Saga 23, host bindings and native packages, which may start from this repo's ABI V1) | feature | a native hook: a way for X_eTaL code to call a function in a native library (`[]S_VO`, reserved in lang-choices QD4 / section 15), with the function's X_eTaL type from the library's descriptor | all | the bridge host `xetal-x` (plan A6): a store routing `ext:` paths of `[]N_PUT` / `[]N_GET` to native functions, text only |
| E2 | landed (vendored X_eTaL f823212) | feature | `.xtlm` macro libraries (MC10-MC13, X_eTaL Saga 19); same as `../X_eTaL-libraries` ask X1 | the binding macro (`ffi:b_ind<`, Saga 8) | none needed now |
| E3 | landed (X_eTaL 8c49bd8: the crate `xetal-command`, steps 145 and 165) | feature | the CLI as a library: a host entry point (`xetal_cli::run_with(args, store)` or similar) so a host can be `xetal` with its own store | the bridge host `xetal-x` | none needed now: `xetal-x` parses and runs through `xetal-command` and repeats only the binary's short `main` with the store wrapped |
| E4 | partly landed (X_eTaL v0.1.0: `[]TS`, the time as year month day hour minute second millisecond, and `[]D_L`, a delay; no monotonic or CPU clock yet) | feature | the time: a quad giving the current time (APL's quad TS) and a monotonic or CPU clock (quad AI) | clock (and anything timing itself) | the clock extension, through the bridge |
| E5 | landed (X_eTaL 8c49bd8, D166: the facade's line, then "called from the program here" with the program's line) | feature | an error raised inside a library function also names the program line that called it (a call trace) | every facade (errors point at the function's signature line in the facade, not the caller) | none needed now; the extension's message still names the function |
| E6 | landed (vendored X_eTaL f823212) | bug | `xetal --draw DIR run FILE` fails (clap takes `run` as the SCRIPT argument and FILE as a subcommand); `xetal run --draw DIR FILE` works | the sqlite notebook's test and every demo recipe | `--draw` written after `run` |
| E7 | landed (X_eTaL d284a8c; re-checked: a facade without `l:package` types as a library) | bug | `xetal type FILE` calls a library whose `l:` names all come from macro calls a program (`library-name-in-program`): it decides before expanding | every facade written with the binding macro | one literal export per facade, `l:package := "NAME"` |
| E8 | landed (X_eTaL d284a8c; re-checked: `quakes.xtl` importing `Quakes` no longer finds itself) | bug | a program `quakes.xtl` importing a library `Quakes` (`"q:" u_se< "Quakes"`) on a case-insensitive file system (macOS) finds the program itself and reports `import-cycle: quakes.xtl -> quakes.xtl`; wanted: library lookup matches the name's case exactly, or the error says the library resolved to the importing file | demos with a library beside them (audio, http) | name the library differently (`Instruments.xtl`, `Seismic.xtl`) |
| E9 | appears fixed (re-checked 2026-11-15 at X_eTaL 8c49bd8: Endless.xtl without its workaround types the same in 10 runs; not traced to a commit) | bug | `xetal type` (and so `xetal doc`) infers different types for the same file from run to run: free type variables or `Int` for the same tuple part, extra `Num e` constraints. Integer literals' types are settled in an order that varies (a hash map's?), so a doc page and the search index change on every build | scene's shared demo libraries (Endless.xtl, Play.xtl) and the demos that use them | kept until the doc pages have stayed stable: the masks made Int for certain, and `scripts/check-pages.sh` ignoring lines that differ only in an inferred type |
| E10 | landed (re-checked 2026-11-15 at X_eTaL 8c49bd8: `xetal doc --json` of `(a, b) := pair` lists no `%1`) | bug | `xetal doc` lists the hidden value a top-level tuple pattern binds -- `(a, b) := pair` shows a value `%1` (drawn in the error color) beside `a` and `b`, on the file's page, its source page and the index; `xetal type` does not list it | the scene demos that unpacked their final state (voxels-dig, -water, -rubik-buttons, -rubik-solve) and the cube cross-check | none needed now; the accessor functions written for it stay, as they read well |
| E11 | open (found 2026-11-04 at X_eTaL d284a8c; re-checked 2026-11-15 at 8c49bd8: the example below still takes 11.4 s) | speed | scalar selection copies the entire array, making a clock-by-clock replay quadratic in trace length | blender physical Nano 1K execution | partition into 64-row chunks and select within each bounded chunk |
| E12 | open (found 2026-11-14 at X_eTaL d284a8c; for X_eTaL-gpu as much as X_eTaL) | speed | array kernels on the browser's GPU: an element-wise program over a large array (a stencil by rotates, a function of a coordinate grid) lowered through XIR to WGSL, the array kept on the device between frames and shared with the renderer | none here; the snow world (snowflow_demo-fork), which this repository supports | the fork keeps its fields small (17 by 17 vertices a chunk) and plans a 256 square snow layer; nothing at the reference's 2048 square |
| E13 | open (found 2026-11-15 at X_eTaL 8c49bd8) | feature | a fix for `rebind` (a name is bound once, M1): `xetal lint --fix` or `xetal migrate` rewriting the sure cases -- an effect result nothing reads to `_ :=`, and a value grown line by line (`t := t c_at piece`) to one expression | every program written before M1: 890 lines in 83 files here | done by hand with three throwaway scripts; no `!` variable was needed |

2026-11-15, X_eTaL 8c49bd8 pinned (the user: October 30 or newer, then
the latest; from d284a8c, 283 commits): E3 and E5 landed and their
workarounds are gone (`xetal-x` uses `xetal-command`; 25 error goldens
now show the calling line). E10 landed. E9 appears fixed; its
workaround stays for now. E1, E4's monotonic clock, E11 and E12 are
still open. E13 added: M1 (a name is bound once, X_eTaL step 091) broke
890 lines here and no tool rewrites them. The newer `xetal doc` also
warns `doc-unattached` on 14 `##` blocks here that head a section and
document no definition: not an ask, they should be `###` or comments.

2026-11-14, the snowflow study (plan, Saga 15 step 26): E12 added.
E1 gains a concrete workload from snowflow_demo-fork
(its docs/xetal-asks.md): an N by 3 Float coordinate array and a T by 3
Int index array kept as one mesh, published from inside an event loop,
with malformed shapes, non-finite numbers and bad indices refused and
the caller named. Until E1 the fork reads typed values through its own
adapter over the output hook; steps 27 and 28 here make that a shared
crate, which is still a workaround and is named as one.

2026-10-08, X_eTaL d284a8c pinned (the user: the newer doc site): E7 and E8 landed; `l:package` is kept in each facade, as it names the native package on the doc pages; libraries' private functions are now `h:` (`xetal migrate`). E1, E3 and E5 still open.

2026-10-05, X_eTaL v0.1.0 (512b3ee) pinned: E4 partly landed (`[]TS`, `[]D_L`; the monotonic clock still needs the clock extension); E1, E3, E5 and E7 still open (E7 re-checked: a facade without `l:package` is still called a program). Macros became hygienic (MC30): an expansion shows `g1:` names.

2026-10-05, later: every facade is now written with the binding macro
(E2 in use); E7 stays open, worked around in each facade.

2026-10-05: X_eTaL f823212 vendored, with E2 (`.xtlm`) and E6
(`--draw` before the subcommand) landed; every golden passed unchanged.
Earlier, swept 2026-10-03 (twice) against upstream X_eTaL (84 commits past the
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

Landed as the crate `xetal-command` (X_eTaL 8c49bd8); `xetal-x` now uses
it and the paragraph above describes the workaround that was.

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
does `XETAL_DRAW=out xetal run prog.xtl`. Workaround (until f823212): `--draw` after `run`. Landed: options may
now come before or after the subcommand; the recipes keep `run --draw`,
which is still valid.

### E7: `xetal type` and macro-written libraries

A library whose exports are all written by macros runs fine when a
program imports it, but `xetal type` refuses it:

```
# Lib.xtl
"g:" u_se< "Gen"
"t_wice" g:d_ef< "y * 2"        # expands to l:t_wice := { ... }
```

```
xetal type Lib.xtl
error[library-name-in-program]: l: names exist only inside a library; import it with an alias
```

`is_library` (xetal-program, library.rs) looks for an `l:` token in the
source before macros expand, finds none, and treats the file as a
program; then the expansion's `l:` names are an error. Wanted: decide
after expansion (or treat a file whose expansion defines `l:` names as
a library). Workaround: each facade written with the binding macro
keeps one literal export, `l:package := "NAME"` (the native package
behind it), which is also useful.

### E8: a library named like the program, on a case-insensitive file system

On macOS, `demos/quakes.xtl` with `"q:" u_se< "Quakes"` beside
`demos/Quakes.xtl`: the two names are one file, so writing one
replaced the other, and the import resolved to the program itself:

```
error[import-cycle]: import cycle: demos/Quakes.xtl -> demos/Quakes.xtl at demos/Quakes.xtl:8:1
```

Hit twice here (the synthesizer's `Synth.xtl`, the earthquake
report's `Quakes.xtl`). Wanted: `u_se<` matches the file name's case
exactly (or warns when the case differs), or the error says the
library resolved to the importing file. Workaround: libraries named
unlike any program beside them (`Instruments.xtl`, `Seismic.xtl`).

### E9: inferred types vary from run to run

`xetal type` prints different types for the same file on different
runs. Ten runs on this repo's `extensions/scene/demos/Play.xtl` (with
`XETAL_PATH` reaching `extensions/scene/demos`, `extensions/scene/lib`
and `lib`) gave four different outputs, for example:

```
l:p_lace : (Any a, Any b, Any c, Any d, Any e) => (((Float, a, Float, Float, b, c), (Int, Int, Box Int, Box Int, Int, Int, Int, Int), (Int, Int, Int, Int), (Int, Int, d, e)), Int) -> ...
l:p_lace : (Any a, Any b, Any c, Num d, Num e, Any f, Any g) => (((Float, a, Float, Float, b, c), (Int, Int, Box Int, Box Int, Int, Int, Int, d), (e, Int, Int, Int), (Int, Int, f, g)), Int) -> ...
```

The parts that vary hold integer literals (a tuple's `0`, `l:pace :=
6`), so the order in which their `Num` types meet `Int` seems to vary.
It started when a state tuple grew to eight parts with several such
literals; a small function alone is stable. The doc site built from it
(`xetal doc --out`) changes on every build. Wanted: the same file
types the same way every run. Workaround: the masks are made Int for
certain (`f_loor f_loat`), which settled `Endless.xtl`; the pages
check ignores lines that differ only in an inferred type. Its filter
passes separate patterns to diff so the same workaround works with
GNU diff on Linux as well as on macOS.

### E10: a top-level tuple pattern's hidden value on the doc pages

```
## a pair, unpacked
pair := (1, 2.5)
(a, b) := pair
a
```

`xetal type` lists `pair`, `a` and `b`. `xetal doc --out DIR` also
lists a value `%1` -- the temporary the pattern binds the whole tuple
to -- on the file's page, its source page and the index, the `%` drawn
in the error color. A reader of the voxel demos' pages asked what
`%85` was. Wanted: the hidden value left out of the doc pages (as
`xetal type` does). Workaround: no top-level tuple patterns here; a
small accessor function takes the part needed
(`u:w_orldOf := { (s, _, _, _) -> s }`, then `s := u:w_orldOf last`).

### E11: scalar selection copies the whole array

The physical Nano 1K replay selects one row for every active clock.
At X_eTaL d284a8c, `s_elect` calls `as_vector`, which clones the entire
array (`xetal-value/src/convert.rs`, `xetal-struct/src/calls.rs`). A loop
over a large trace therefore copies the whole trace on each iteration.
The initial 221,161-clock replay was stopped after seven minutes;
selecting within 64-row chunks completes the same full comparison.

Minimal example of the access pattern:

```xtl
rows := (10000 38) r_eshape o_ffsets 380000
10000 '{ (total, i) -> (total + f_irst i s_elect rows, i + 1) } p_ower (0, 1)
```

Wanted: scalar selection reads only the requested item without copying
unrelated rows. Workaround: `keys-leds-replay.xtl` partitions once into
64-row chunks, selects each chunk once, then selects rows within that
bounded chunk. No upstream source is changed.

### E12: array kernels on the browser's GPU

The snow demo this family wants to write in X_eTaL
(https://github.com/Noniv/snowflow_demo) keeps its deformable snow in a
2048 square state array and rewrites every cell every frame: each cell
relaxes toward its four neighbors, then brushes are added. As an array
program that is rotates and arithmetic, which X_eTaL says directly:

```xtl
d := (2048 2048) r_eshape f_loat o_ffsets (2048 * 2048)
lap := ((1 o_- d) + (-1 o_- d) + (1 o_-_2 d) + (-1 o_-_2 d)) - 4.0 * d
next := d + 0.1 * lap
s_hape next
```

At X_eTaL d284a8c on an M1 Max, `xetal run` takes 1.22 s for this and
0.13 s for the first line alone: 1.1 s for one step, where a frame has
16.7 ms. A 256 square array takes about 10 ms (too short to time well
from the command line). The reference runs the same step as one WGSL
pass every frame; its time was not measured here.

Wanted: the element-wise subset (maps, rotates, reductions, lookups)
lowered through X_eTaL-gpu's XIR to WGSL compute, run in the browser,
with the array resident on the device from one frame to the next and
readable by the renderer without a copy back. Numbers on the device are
f32 and i32: the narrowing is explicit and refused where it would lose
an Int. Results agree with the evaluator within a stated tolerance.
Per-pixel shading (texture reads, screen-space derivatives) is a
separate, later contract and is not part of this ask.

Workaround: none at this size. The fork keeps fields small and runs
them in the evaluator; its third planned saga is this ask.

### E13: a fix for rebind

Since M1 a name without `!` is bound once in its scope, and a program
written before it stops with `error[rebind]` at its first repeat. The
message offers `a!`, which is right only for a variable that changes
over time; in the 890 lines this repository had, none was one. They
were four habits, each with a sure rewrite:

```xtl
t := "a"                    # a value grown line by line ...
t := t c_at "b"
t := t c_at "c"

t := ("a" c_at              # ... is one expression
  "b" c_at
  "c")

n := "f.py" bl:w_rite! t    # an effect's result nothing reads ...
_ := "f.py" bl:w_rite! t    # ... is a discard (M3)

s := f s                    # a step applied to its own result ...
s := g s
s := g f s0                 # ... is a composition

w := a au:w_indow 1024      # one name for two things needs a second name
```

Wanted: `xetal lint --fix` (or `xetal migrate`) doing the first three,
which need no judgment (`c_at`, `c_at_2`, `m_ax` and `m_in` are
associative; the bindings moved are pure), and listing the fourth. The
error could also say "or give the new value its own name" beside `a!`.
The linter reports one `rebind` and stops; listing every one in the
file would let a person see the size of the job.

Workaround: scripts written for this move and thrown away; the goldens,
unchanged, were the proof.
