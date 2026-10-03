//! The shell's run, natively: hello linked statically (as a page links
//! it), its facade served from the store.

use xetal_ext_shell::run::{Linked, install, run};

#[test]
fn a_program_runs_with_the_linked_extension() {
    let functions = install(&[Linked {
        descriptor: xetal_ext_hello::__xetal_extension::descriptor,
        facade: (
            "Hello.xtl",
            include_str!("../../../extensions/hello/lib/Hello.xtl"),
        ),
    }]);
    assert_eq!(functions.len(), 8);
    assert_eq!(functions[3].signature, "Char -> Char");

    let o = run("\"hx:\" u_se< \"Hello\"\nhx:s_hout \"x_etal\"\nhx:s_um 2 3 r_eshape r_ange 6\n");
    assert_eq!((o.out.as_str(), o.err.as_str()), ("X_ETAL\n21.0\n", ""));

    let o = run("\"hx:\" u_se< \"Hello\"\nhx:p_anic @\n");
    assert!(
        o.err
            .contains("extension panicked: hello was asked to panic"),
        "{}",
        o.err
    );

    let o = run("1 +");
    assert!(o.err.starts_with("error["), "{}", o.err);
}
