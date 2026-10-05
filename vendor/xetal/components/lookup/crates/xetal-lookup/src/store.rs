//! Libraries from the store the host installed (`xetal-store`), then
//! the standard libraries built in: the live demo's libraries, where
//! the files are the browser's local storage. A name is `Name.xtl` and
//! `Name.xtlm` in the store (MC11); a path is looked up as written.

use crate::disk::standard;
use crate::found::{Pair, Spec, spec};
use crate::{Found, Libraries};

pub struct StoreLibraries;

impl Libraries for StoreLibraries {
    fn find(&self, spec: &str, from: &str) -> Option<Found> {
        self.find_both(spec, from).0
    }

    fn find_both(&self, name: &str, _from: &str) -> Pair {
        match spec(name) {
            Spec::Library => (stored(name), None),
            Spec::Macros => (None, stored(name)),
            Spec::Name => match (
                stored(&format!("{name}.xtl")),
                stored(&format!("{name}.xtlm")),
            ) {
                (None, None) => standard(name),
                pair => pair,
            },
        }
    }
}

fn stored(path: &str) -> Option<Found> {
    xetal_store::read(path).ok().map(|text| Found {
        key: format!("store:{path}"),
        name: path.into(),
        text,
    })
}
