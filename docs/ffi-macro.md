# The binding macro (`Ffi.xtlm`)

A facade function (in `extensions/NAME/lib/NAME.xtl`) speaks the
`ext:` channel: put each argument, get the reply, read it back as the
type the native function returns. It is the same few lines for every
function, decided by the function's signature, so with X_eTaL's
`.xtlm` macro libraries (ask E2, landed; MC10-MC13) a macro writes
them: every facade here is one line per native function. `lib/Ffi.xtlm` is that macro; `xetal-x` puts `lib/` on
`XETAL_PATH`, so a facade imports it as `"ffi:" u_se< "Ffi"`.

## How a facade reads

```
# Hello.xtl
"ffi:" u_se< "Ffi"

"a_nswer : unit -> int"        ffi:b_ind< "hello/answer"
"a_dd : float float -> float"  ffi:b_ind< "hello/add"
"s_hout : text -> text"        ffi:b_ind< "hello/shout"
"s_um : float -> float"        ffi:b_ind< "hello/sum"
"e_cho : float -> floats"      ffi:b_ind< "hello/echo"
"f_ail : unit -> int"          ffi:b_ind< "hello/fail"
"p_anic : unit -> int"         ffi:b_ind< "hello/panic"
```

The left text is the export's name and its signature in channel
kinds; the right text is the native function, `EXTENSION/FUNCTION`.
Each call stands as a top-level statement (MC12) and expands to one
`l:` definition. `Ffi.xtlm` is its own macro library, so the facade
never uses a macro it defines (MC10).

## Kinds

| Kind | As an argument (put) | As the result (get) | X_eTaL type |
| ---- | -------------------- | ------------------- | ----------- |
| `unit` | no argument: `{ @ -> ... }` | -- | `Unit` |
| `text` | `a []N_PUT "ext:E/F?text"` | `[]N_GET "ext:E/F"` | `Char` |
| `float` | `(f_ormat r_avel f_loat a) []N_PUT "ext:E/F?float=" c_at f_ormat s_hape a` (any rank) | `f_irst n_umbers []N_GET "ext:E/F"` (a scalar) | `Num a => a` in, `Float` out |
| `floats` | as `float` | `v := n_umbers []N_GET "ext:E/F"` then `(f_loor n_umbers []N_GET "ext:E/F?shape") r_eshape v` | `Float`, any rank |
| `int` | `(f_ormat r_avel f_loor a) []N_PUT "ext:E/F?int=" c_at f_ormat s_hape a` | `f_loor f_irst n_umbers []N_GET "ext:E/F"` | `Int` |
| `chars` | `(r_avel a) []N_PUT "ext:E/F?chars=" c_at f_ormat s_hape a` | `v := []N_GET "ext:E/F"` then `(f_loor n_umbers []N_GET "ext:E/F?shape") r_eshape v` | `Char`, any rank |
| `ints` | -- | as `floats`, with `f_loor` | `Int`, any rank |

One argument kind before `->` makes a monadic export (`{ a -> ... }`),
two a dyadic one (`{ a b -> ... }`, left then right), `unit` none.
Every put is bound to a name (`pa :=`, `pb :=`) so it runs as a
statement; the get is the last line.

## The expansion, exactly

`"s_hout : text -> text" ffi:b_ind< "hello/shout"` becomes

```
l:s_hout := { a ->
  pa := a []N_PUT "ext:hello/shout?text"
  []N_GET "ext:hello/shout"
}
```

and `"e_cho : float -> floats" ffi:b_ind< "hello/echo"` becomes

```
l:e_cho := { a ->
  pa := (f_ormat r_avel f_loat a) []N_PUT "ext:hello/echo?float=" c_at f_ormat s_hape a
  v := n_umbers []N_GET "ext:hello/echo"
  (f_loor n_umbers []N_GET "ext:hello/echo?shape") r_eshape v
}
```

`docs/ffi/Hello.xtl` is the whole of hello's facade expanded by these
rules, written by hand before the macro existed, so the expansions
were known to be right first. Its tests stay as the macro's reference:

- `hello-ffi-types` (reg-rs): its types equal the facade's
  (`hello-types`);
- `hello-ffi-expansion` (reg-rs): hello's tour, with this file in place
  of `lib/Hello.xtl` (first on `XETAL_PATH`), prints the tour's golden.

## The macro

`lib/Ffi.xtlm`, about 60 lines of ordinary X_eTaL string work: split
the signature into words (`p_artition` on spaces), take the name, the
argument kinds and the result kind, and join the lines of the table
above with `c_at`. Its private helpers (`v_alue`, `q_uery`, `p_ut`,
`g_et`) each turn one kind into its text.

Tested by reg-rs in the hello extension, against the goldens:

- `hello-ffi-macro`: `xetal-x expand tests/ffi-hello.xtl` (hello's seven
  exports as signature lines) gives exactly the definitions of
  `docs/ffi/Hello.xtl`, once hygiene's fresh names are read as the
  names they stand for (X_eTaL v0.1.0 renames what a macro binds,
  `a` to `g1:a`, so it cannot capture a name of the caller's);
- `hello-ffi-macro-types`: their types equal the hand-written facade's
  (`hello-types`), plus `l:package`;
- `hello-ffi-macro-tour`: the tour, with the macro-written facade in
  place of `lib/Hello.xtl`, prints the tour's golden.

A facade written this way keeps one literal export,
`l:package := "NAME"`: `xetal type` decides whether a file is a library
before macros expand (ask E7).

## The facades

All six facades -- hello, clock, sqlite, canvas, scene and audio --
are written this way. Every function's type is the one the
hand-written facade had (each `NAME-types` golden gained only
`l:package : Char`), and every demo prints what it printed before:
programs did not notice. One function is still written by hand,
clock's `t_ime`, which times a function argument and so has no kind
here. An error raised by a native function now points at that
function's signature line in the facade (`./lib/Sqlite.xtl:14:1`),
not at a shared helper.

When X_eTaL calls native code itself (E1, Saga 23), only `Ffi.xtlm`
changes: the facades keep their lines, and the expansion calls the
native hook instead of the `ext:` channel.
