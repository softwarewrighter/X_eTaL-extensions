# X_eTaL-extensions tasks. Recipes call scripts/*.sh, which hold the
# logic and work without just too. `just` alone lists the recipes.

set positional-arguments

# List the recipes
default:
    @just --list

# Snapshot a committed ref of ../X_eTaL into vendor/xetal/ (default HEAD); commit it on its own
vendor ref="HEAD":
    scripts/vendor-xetal.sh "$1"

# Build the vendored xetal CLI into target/xetal/
xetal:
    @scripts/build-xetal.sh

# The vendored X_eTaL: what was vendored (VENDORED) and the binary's version
xetal-version:
    @cat vendor/xetal/VENDORED
    @"$(scripts/build-xetal.sh)" --version | head -1

# Evaluate an expression with the vendored xetal: just eval "'+ r_/ 1 2 3"
eval expr:
    @"$(scripts/build-xetal.sh)" eval -e "$1"

# Check the vendored X_eTaL: the CLI builds, answers and imports a standard library
check-vendor:
    scripts/check-vendor.sh

# Build every crate and extension (the extensions' shared libraries land in target/debug/)
build:
    cargo build --workspace

# This repo's Rust: fmt, clippy (warnings are errors), tests
check-rust:
    scripts/check-rust.sh

# Run the Rust tests (optionally one crate): just test xetal-ext-abi
test crate="":
    @if [ -n "$1" ]; then cargo test -p "$1"; else cargo test --workspace; fi

# Run a program with the bridge host and every extension here: just run-x FILE [ARGS]
run-x file *args:
    @cargo build -q --workspace
    @target/debug/xetal-x --ext extensions run "$@"

# The loaded extensions and their functions
ext-list:
    @cargo build -q --workspace
    @target/debug/xetal-x --ext extensions --ext-list

# xetal-x matches the vendored xetal on every vendored demo
check-xetal-x:
    scripts/check-xetal-x.sh

# Run one extension's recipe: just ext hello test (see extensions/NAME/justfile)
ext name *args:
    @just -f "extensions/$1/justfile" "${@:2}"

# Every extension's reg-rs tests (and their layout)
test-exts:
    scripts/test-exts.sh

# Start an extension from templates/extension: just new-ext sqlite sq "SQLite databases"
new-ext name alias what:
    scripts/new-ext.sh "$1" "$2" "$3"

# The live pages' crates: fmt, clippy, native tests, wasm32 build
check-web:
    scripts/check-web.sh

# Build the live site into pages/ (commit it; a push publishes it)
pages:
    scripts/build-pages.sh

# Preview pages/ as GitHub Pages serves it, at http://127.0.0.1:8732/X_eTaL-extensions/
serve-pages:
    @rm -rf target/serve && mkdir -p target/serve && ln -s "$PWD/pages" target/serve/X_eTaL-extensions
    @echo "http://127.0.0.1:8732/X_eTaL-extensions/"
    @python3 -m http.server 8732 --directory target/serve

# A fresh user's first run: clone, build, run release 1's programs (just walkthrough [URL])
walkthrough *url:
    scripts/walkthrough.sh "$@"

# docs/status.md, what works today, regenerated
status-doc:
    scripts/status.py

# List the demos (EXT NAME and what each shows)
demos:
    @scripts/demos.sh

# Run a demo: just demo sqlite notebook (extra args go to xetal-x run, e.g. --echo)
demo ext name *args:
    @scripts/demos.sh "$@"

# Record the demo videos (vhs, ffmpeg, gif2webp; headless, nothing on screen): just videos [EXT]
videos *ext:
    scripts/videos.sh "$@"

# The full pre-commit gate
gate:
    scripts/gate.sh

# Show the agentrail saga state and the current step
status:
    agentrail status

# Open the saga plan
plan:
    @cat docs/plan.md
