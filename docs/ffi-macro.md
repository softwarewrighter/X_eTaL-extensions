# The binding macro (`Ffi.xtlm`)

Every extension's facade (`extensions/NAME/lib/NAME.xtl`) is hand-written
code that speaks the `ext:` channel: put each argument, get the reply,
read it back as the type the native function returns. It is the same
few lines for every function, decided by the function's signature.
Once X_eTaL has `.xtlm` macro libraries (ask E2, X_eTaL Saga 19,
decisions MC10-MC13), a macro writes those lines: a facade becomes one
line per native function. This page is the design, ready for that day;
X_eTaL-extensions is one of the three repositories waiting on `.xtlm`
(research4).

## How a facade will read

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
rules, written by hand. It is tested today, so the expansions are known
to be right before the macro exists:

- `hello-ffi-types` (reg-rs): its types equal the hand-written
  facade's (`hello-types`);
- `hello-ffi-expansion` (reg-rs): hello's tour, with this file in place
  of `lib/Hello.xtl` (first on `XETAL_PATH`), prints the tour's golden.

## The macro (sketch)

`Ffi.xtlm` defines one macro, a function of the two texts (MC10):

```
# Ffi.xtlm -- generate a facade function from a native function's signature.
# Import: "ffi:" u_se< "Ffi"
m:b_ind< := { sig target ->
  # sig: "NAME : KIND [KIND] -> KIND"; target: "EXT/FUNCTION"
  # 1. split sig at " : " and " -> " (the name, the argument kinds, the result kind)
  # 2. one put line per argument kind (the table above), names pa, pb
  # 3. the result lines for the result kind
  # 4. "l:" NAME " := { " args " ->" newline, the lines, "}"
  ...
}
```

Its body is ordinary X_eTaL string work (splitting on spaces,
concatenating with `c_at`), written and tested when `.xtlm` lands --
against the goldens above: `xetal --expand` of the macro-based
`Hello.xtl` (ask X2 in X_eTaL-libraries) must give `docs/ffi/Hello.xtl`'s
definitions, and its types must stay `hello-types`.

## The day `.xtlm` lands

1. Refresh the vendored X_eTaL (its own commit).
2. Write `lib/Ffi.xtlm` (in a shared directory every facade can import,
   for example `lib/` at the repository root, put on `XETAL_PATH` by
   `xetal-x`), test its expansions against `docs/ffi/Hello.xtl`.
3. Rewrite each facade (hello, clock, sqlite, canvas) as signature
   lines; every facade's `NAME-types` golden must not change, and every
   demo's golden neither (the proof that programs did not notice).
4. Later, when X_eTaL calls native code itself (E1, Saga 23), only
   `Ffi.xtlm` changes: the facades keep their lines, the expansion
   calls the native hook instead of the `ext:` channel.
