# macros

X_eTaL's .xtlm macro libraries landed upstream (ask E2; MC10-MC13,
MC23), with the --draw fix (E6). This saga brings them in and makes
"macros extend the language" real in this repository: the binding
macro Ffi.xtlm (docs/ffi-macro.md) writes every facade function from a
one-line signature.

## Steps

1. vendor-refresh -- vendor X_eTaL HEAD (own commit), re-run every
   golden, fix or rebase on purpose, drop the --draw workaround.
2. ffi-macro -- Ffi.xtlm (shared, on XETAL_PATH via xetal-x), its
   expansions tested against docs/ffi/Hello.xtl (xetal expand).
3. facades -- each facade rewritten as signature lines where the kinds
   fit; every types golden and demo golden unchanged.
4. macros-release -- docs, status, asks, retrospective.
