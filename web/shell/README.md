# The live pages' shell

Every live page runs an X_eTaL program in the browser with native
extensions compiled to WebAssembly and linked in. This crate
(`xetal-ext-shell`) is what the pages share; adapted from
X_eTaL-demos' microscope (copied, not depended on).

| Module | What it gives a page |
| ------ | -------------------- |
| `run` | `install(&[Linked])` registers statically linked extensions (descriptor and facade source) and installs the store that serves their facades and `ext:` calls (`xetal-ext-bridge`); `run(src)` runs a program with the vendored `xetal-play` and times it |
| `source` | `code(src)` and `block(src)`: X_eTaL drawn decorated, as X_eTaL renders it |
| `chrome` | `header` (logo, title, lede), `panel`, `footer` (copyright, license, repository, all extensions, the vendored X_eTaL commit, the build) |
| `playground` | `Playground`: the page body -- the program (editable, Run and Reset), its decorated source, its output or error, the native functions linked in, the command-line equivalent |

`shell.css` is the stylesheet (light and dark, this repository's red);
a page links it with
`<link data-trunk rel="css" href="../../../web/shell/shell.css" />`,
and the logo and the red favicon from `images/`.

## A page

An extension's page is `extensions/NAME/web/`, a Cargo workspace of
its own (pages link extensions with their `static` feature, which must
never reach the root workspace's shared libraries):

- `Cargo.toml`: the shell, the extension's crate with
  `features = ["static"]`, yew;
- `index.html`: trunk links to the binary, the logo, the favicon,
  `shell.css`;
- `src/main.rs`: `install` the extensions, then render a `Playground`
  with a demo program (`include_str!("../../demos/NAME.xtl")`);
- `tests/`: the page's program run natively against the golden.

`scripts/check-web.sh` (in the gate) checks every page crate natively
and for wasm32. Page crates build into `target/web/` (each has a
`.cargo/config.toml` saying so): they compile the same crates as the
root workspace with other features, and one shared `deps/` directory
mixes the two builds.
