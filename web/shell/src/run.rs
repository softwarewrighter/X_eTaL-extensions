//! Installing extensions and running programs against them.

use std::sync::Arc;

use xetal_ext_bridge::ExtStore;
use xetal_ext_loader::abi::ExtensionDescriptorV1;
use xetal_ext_loader::{FunctionInfo, Registry};
use xetal_store::Memory;

/// One statically linked extension: its descriptor function and its
/// facade (file name and source).
pub struct Linked {
    pub descriptor: fn() -> *const ExtensionDescriptorV1,
    pub facade: (&'static str, &'static str),
}

/// Registers the extensions, installs the store that serves their
/// facades and `ext:` calls (files in memory otherwise), and returns
/// every function, for the page to list.
///
/// # Panics
///
/// When a linked descriptor is invalid: a build error, not a user's.
pub fn install(linked: &[Linked]) -> Vec<FunctionInfo> {
    let mut registry = Registry::new();
    let mut facades = Vec::new();
    for l in linked {
        registry
            .load_static(l.descriptor)
            .expect("a linked extension's descriptor is valid");
        facades.push(l.facade);
    }
    let functions = registry
        .extensions()
        .iter()
        .flat_map(|e| registry.functions(e))
        .collect();
    // the binding macro every facade is written with (lib/Ffi.xtlm)
    let mut store = ExtStore::new(Memory::default(), registry)
        .with_facade("Ffi.xtlm", include_str!("../../../lib/Ffi.xtlm"));
    for (name, source) in facades {
        store = store.with_facade(name, source);
    }
    xetal_store::install(Arc::new(store));
    functions
}

/// What a run printed, its error, and how long it took.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    pub out: String,
    pub err: String,
    pub millis: f64,
}

/// Runs `src` (seed 1) with the installed extensions.
pub fn run(src: &str) -> Outcome {
    let start = now();
    let run = xetal_play::run(src, 1);
    Outcome {
        out: run.out,
        err: run.err,
        millis: now() - start,
    }
}

/// Milliseconds from the browser's clock (0 when not in a browser).
pub fn now() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map_or(0.0, |p| p.now())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}
