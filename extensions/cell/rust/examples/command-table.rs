//! Prints the command table as markdown, for docs/README.md (a test
//! fails when the page's table is not this): `just command-table`.

fn main() {
    print!("{}", xetal_ext_cell::command::markdown());
}
