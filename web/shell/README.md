# The live pages' shell

Every live page runs an X_eTaL program in the browser with native
extensions compiled to WebAssembly and linked in. This crate
(`xetal-ext-shell`) is what the pages share; adapted from
X_eTaL-demos' microscope (copied, not depended on).

| Module | What it gives a page |
| ------ | -------------------- |
| `run` | `install(&[Linked])` registers statically linked extensions (descriptor and facade source) and installs the store that serves their facades and `ext:` calls (`xetal-ext-bridge`); `run(src)` runs a program with the vendored `xetal-play` and times it |
| `play` | a game kept running: `Files` (what the program reads, (name, text) pairs), `Game` (`start`, `event(line)`, `state`, `take_printed`, `take_picture`, `error`, `stop`), `scene(svg)` (the board's calls a picture carries) |
| `page` | one game for a page: `Page` (program, files, linked extensions) and `game_page!(PAGE)`, which exports what `extensions/three/web/static/game.js` drives |
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

## A game's page

A game is an X_eTaL program that waits for events (`[]E_VENT`) and
shows a picture after each (`[]S_HOW`: an SVG document whose
`<metadata id="scene">` holds the board player's calls). Its page
keeps the program running and gives it a line an event; no host code
knows a game's rules. The whole of `src/main.rs` (the crate depends
on this one and on `wasm-bindgen`):

```rust
use xetal_ext_shell::page::Page;

const PAGE: Page = Page {
    program: "play.xtl",
    // the program, the libraries it imports, the data it reads, each
    // under the path X_eTaL asks for
    files: &[("play.xtl", include_str!("../../games/walk/play.xtl"))],
    // extensions linked in (run::Linked), if the game calls any
    linked: &[],
};

xetal_ext_shell::game_page!(PAGE);

fn main() {}
```

`game_page!` exports `start(seed)`, `event(line)` (true once the game
is over), `picture()` and `printed()` (what was shown and printed
since last asked), `error()`, `over()`, `stop()`, `files()` and
`rendered(name)` (the sources, X_eTaL drawn decorated). The page's
script hands them to `game.js`, with every choice of the game's as
setup:

```js
import { game, named } from './ext/game.js';

const play = game(window.wasmBindings, {
  view: document.getElementById('view'),   // the board is drawn here
  flat: document.getElementById('flat'),   // the SVG picture goes here
  tick: 0.05,
  tickLine: (seconds) => `tick ${seconds}`,
  keyLine: named({ ' ': 'space', ArrowUp: 'up' }, (name) => `key ${name}`),
  clickLine: (x, y) => `click ${x.toFixed(1)} ${y.toFixed(1)}`,
  board: { models: './models/' },
  onPrinted: (text) => { log.textContent += text; },
  onOver: (error) => { log.textContent += error; },
});
play.start();
```

`game.js` runs the clock (a late timer is caught up a few ticks at
most; a hidden tab gets none), turns keys and clicks into lines
(clicks in the view's units, from the board or from the flat
picture), and after each event draws the picture's scene with
`board.js`; where there is no WebGL, or a picture has no scene, the
flat picture is shown whole. While the board draws, the picture's
element has the class `solid`, for the page's stylesheet to hide what
the board draws (`#flat.solid .flat { display: none }`).

Natively (tests, tools) the same game is `play::Game`:

```rust
let files = Files::new([("play.xtl", source)]);
let mut game = Game::start(&files, &[], "play.xtl", 1);
game.event("tick 0.05");
let picture = game.take_picture();   // Some(svg) when it showed one
let printed = game.take_printed();
```

The store is one for the process: a game installs its own again at
each event, so a page may run other programs between events.
