# The bridge: calling extensions from X_eTaL today

X_eTaL cannot yet call native code itself (`docs/xetal-asks.md`, E1).
Until it can, programs that use extensions run with `xetal-x`, the
vendored `xetal` CLI -- every subcommand, option and message the same
-- whose store sends paths starting `ext:` to native extensions. It is
APL's shared-variable idea: the program writes to a name, the partner
reads it and answers through the same name. Facade libraries
(`extensions/NAME/lib/NAME.xtl`) hide the channel behind ordinary typed
functions, so a program only imports the facade:

```
"hx:" u_se< "Hello"
hx:s_hout "x_etal"
```

## Running

```sh
xetal-x --ext extensions run prog.xtl      # every extension under extensions/
xetal-x --ext extensions/hello run prog.xtl
XETAL_EXT_PATH=extensions xetal-x run prog.xtl
xetal-x --ext extensions --ext-list        # what is loaded, with X_eTaL types
just run-x prog.xtl                        # the same, from the repository root
```

`--ext DIR` (repeatable) and `--ext-list` come before the subcommand; a
directory is a package (it has an `extension.toml`) or holds packages.
A package's library is looked for under its `native/<platform>/`, then
beside `xetal-x` (where Cargo builds the workspace's extensions).

## The protocol

Every path is `ext:EXTENSION/FUNCTION`, optionally with a query.

| Operation | Path | Meaning |
| --------- | ---- | ------- |
| `t []N_PUT p` | `ext:E/F?text` (or no query) | the next argument: the text `t` |
| `t []N_PUT p` | `ext:E/F?chars=SHAPE` | the next argument: a Char array of that shape, `t` its ravel |
| `t []N_PUT p` | `ext:E/F?int=SHAPE`, `?float=SHAPE`, `?bool=SHAPE` | the next argument: numbers (or 1/0), `t` their ravel separated by spaces (`f_ormat r_avel x`) |
| `[]N_GET p` | `ext:E/F` | call `F` with the arguments put since its last call; the reply as text |
| `[]N_GET p` | `ext:E/F?shape` | the last reply's shape (empty for a scalar; a text's length) |
| `[]N_GET p` | `ext:E/F?kind` | the last reply's kind: `text`, `chars`, `int`, `float` or `bool` |

SHAPE is the axes separated by spaces (`f_ormat s_hape x`); empty or
absent means a scalar. Arguments are passed left then right; a
function of arity 0 is called with no puts.

A reply is text as it is, or numbers (Bools as 1 and 0) ravelled and
separated by spaces. Floats are written in the shortest form that reads
back as the same number (`1.0`, `0.30000000000000004`, `1e-20`), which
`n_umbers` reads; Ints read back with `f_loor n_umbers`. A facade
reshapes with the `?shape` reply:

```
q := (f_ormat r_avel m) []N_PUT "ext:hello/echo?float=" c_at f_ormat s_hape m
e := n_umbers []N_GET "ext:hello/echo"
(f_loor n_umbers []N_GET "ext:hello/echo?shape") r_eshape e
```

## Errors

Anything that goes wrong is an ordinary X_eTaL error at the `[]N_PUT`
or `[]N_GET`, naming the path:

```
error[io]: []N_GET: ext:hello/fail: hello was asked to fail at 76..100
error[io]: []N_GET: ext:hello/add: hello/add takes 2 arguments, given 1 at 98..121
error[io]: []N_PUT: ext:hello/add?int: "1.5" is not an Int at 73..106
error[io]: []N_GET: ext:hello/panic: extension panicked: hello was asked to panic at 75..100
```

An unloaded extension, an unknown function, a bad query or shape, an
argument whose text is not its kind, the wrong number of arguments, a
failure or a panic inside the extension: each stops the program with
such a message. A panic is contained in the extension; the process
goes on.

## Cost and limits

Every argument and reply crosses as text: two store calls and a
formatting round trip per call. Measured by the clock extension's
`demos/bridge-cost.xtl` (Apple M-series laptop, release build): about
220,000 calls per second, 2.6 million Floats per second out and back.
A 100,000-element sum takes about 21 ms in X_eTaL and about 20 ms in
Rust over the bridge, most of it formatting numbers as text: the
bridge pays off for work that is heavy per element (an SVD, a regular
expression, an image codec), not for a sum. In a browser (the clock's
live page, Chrome) the channel is slower -- about 16,000 calls and 1.7
million Floats per second -- and X_eTaL's own sum faster (about 14 ms). Values are bounded by
ABI V1 (64 Mi elements).

## In code

The channel is `crates/xetal-ext-bridge`: `ExtStore` wraps a host's
store and a loader `Registry`. `xetal-x` wraps the CLI's store with it;
a browser page wraps `xetal-play`'s memory store and also registers
each facade's source (`ExtStore::with_facade("Hello.xtl", ...)`), since
there `u_se<` reads libraries from the store. The protocol itself is
`xetal_ext_bridge::protocol`.

## What replaces it

When X_eTaL has a native hook (E1), each facade calls the native
function directly, the `ext:` channel goes, and programs do not change.
`xetal-x` itself also compiles the vendored CLI's modules by path
(E3); a CLI library entry point would remove that.
