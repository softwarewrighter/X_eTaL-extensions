//! The bridge in a store-only host: the vendored xetal-play (what the
//! browser runs) with hello linked statically, its facade served from
//! the store, gives the same output as xetal-x does on the command line.

use std::sync::Arc;

use xetal_ext_bridge::ExtStore;
use xetal_ext_loader::Registry;
use xetal_store::Memory;

const FACADE: &str = include_str!("../../../extensions/hello/lib/Hello.xtl");
// the binding macro the facade is written with
const FFI: &str = include_str!("../../../lib/Ffi.xtlm");
const TOUR: &str = include_str!("../../../extensions/hello/demos/tour.xtl");
const TOUR_OUT: &str = include_str!("../../../extensions/hello/tests/hello-demo-tour.out");

fn install() {
    let mut registry = Registry::new();
    registry
        .load_static(xetal_ext_hello::__xetal_extension::descriptor)
        .unwrap();
    let store = ExtStore::new(Memory::default(), registry)
        .with_facade("Hello.xtl", FACADE)
        .with_facade("Ffi.xtlm", FFI);
    xetal_store::install(Arc::new(store));
}

#[test]
fn hello_tour_runs_as_on_the_command_line() {
    install();
    let run = xetal_play::run(TOUR, 1);
    assert_eq!(run.err, "");
    assert_eq!(run.out, TOUR_OUT);

    // a native failure is the program's error, as on the command line
    let run = xetal_play::run("\"hx:\" u_se< \"Hello\"\nhx:f_ail @\n", 1);
    assert!(
        run.err.contains("ext:hello/fail: hello was asked to fail"),
        "{}",
        run.err
    );

    // other files still reach the wrapped store
    let run = xetal_play::run("n := \"x\" []N_PUT \"note.txt\"\n[]N_GET \"note.txt\"\n", 1);
    assert_eq!(run.out, "x\n");
}
