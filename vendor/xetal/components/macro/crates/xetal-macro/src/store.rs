//! Libraries from the store the host installed (`xetal-store`), then
//! the standard libraries built in: the live demo's libraries, where
//! the files are the browser's local storage. A name is `Name.xtl` in
//! the store; a path (with `/` or ending in `.xtl`) is looked up as
//! written.

use crate::fs::standard;
use crate::{Found, Libraries};

pub struct StoreLibraries;

impl Libraries for StoreLibraries {
    fn find(&self, spec: &str, _from: &str) -> Option<Found> {
        let path = match spec.contains('/') || spec.ends_with(".xtl") {
            true => spec.to_string(),
            false => format!("{spec}.xtl"),
        };
        match xetal_store::read(&path) {
            Ok(text) => Some(Found {
                key: format!("store:{path}"),
                name: path,
                text,
            }),
            Err(_) => standard(spec),
        }
    }
}
