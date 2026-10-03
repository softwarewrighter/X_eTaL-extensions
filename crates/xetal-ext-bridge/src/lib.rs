//! The `ext:` channel between X_eTaL and native extensions.
//!
//! [`ExtStore`] wraps a host's store (the CLI's disk, the browser's
//! memory): paths starting `ext:` reach the extensions of a
//! [`Registry`] by the text protocol of `protocol` (docs/bridge.md),
//! facade sources are served by file name, and everything else goes to
//! the wrapped store. `xetal-x` and the live pages both use it.

use std::collections::HashMap;
use std::sync::Mutex;

use xetal_ext_loader::{Registry, Value};
use xetal_store::Store;

use crate::protocol::Request;

pub mod protocol;

/// The path prefix that reaches extensions.
pub const PREFIX: &str = "ext:";

/// A host's store, with `ext:` paths sent to extensions (the protocol
/// is in `protocol.rs` and docs/bridge.md) and the extensions' facades
/// served by file name (`Hello.xtl`), for hosts whose programs find
/// libraries through the store (the browser's `xetal-play`).
pub struct ExtStore<S> {
    inner: S,
    registry: Registry,
    facades: HashMap<String, String>,
    state: Mutex<State>,
}

/// Per `EXT/FN`: the arguments put since the last call, and the last
/// call's reply.
#[derive(Default)]
struct State {
    pending: HashMap<String, Vec<Value>>,
    last: HashMap<String, Value>,
}

impl<S: Store> ExtStore<S> {
    pub fn new(inner: S, registry: Registry) -> Self {
        Self {
            inner,
            registry,
            facades: HashMap::new(),
            state: Mutex::new(State::default()),
        }
    }

    /// Serves `source` as the file `name` (a facade, `Hello.xtl`), so
    /// `u_se<` finds it in a host that reads libraries from the store.
    #[must_use]
    pub fn with_facade(mut self, name: impl Into<String>, source: impl Into<String>) -> Self {
        self.facades.insert(name.into(), source.into());
        self
    }

    /// The registry the store calls into.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Checks that `ext/function` is loaded; the key for its state.
    fn target(&self, ext: &str, function: &str) -> Result<String, String> {
        if !self.registry.extensions().contains(&ext) {
            return Err(format!("no extension {ext} is loaded (xetal-x --ext DIR)"));
        }
        if self.registry.help(ext, function).is_none() {
            return Err(format!("extension {ext} has no function {function}"));
        }
        Ok(format!("{ext}/{function}"))
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn ext_get(&self, path: &str) -> Result<String, String> {
        match protocol::parse(path)? {
            Request::Call { ext, function } => {
                let key = self.target(ext, function)?;
                let args = self.state().pending.remove(&key).unwrap_or_default();
                let value = self
                    .registry
                    .call(ext, function, &args)
                    .map_err(|e| e.to_string())?;
                let text = protocol::reply(&value);
                self.state().last.insert(key, value);
                Ok(text)
            }
            Request::Shape { ext, function } => {
                let key = self.target(ext, function)?;
                self.state()
                    .last
                    .get(&key)
                    .map(protocol::shape)
                    .ok_or_else(|| format!("{key} has not been called"))
            }
            Request::KindOf { ext, function } => {
                let key = self.target(ext, function)?;
                self.state()
                    .last
                    .get(&key)
                    .map(|v| protocol::kind(v).name().to_owned())
                    .ok_or_else(|| format!("{key} has not been called"))
            }
            Request::Argument { .. } => Err("an argument is put, not got".into()),
        }
    }

    fn ext_put(&self, path: &str, text: &str) -> Result<(), String> {
        let (ext, function, value) = match protocol::parse(path)? {
            Request::Call { ext, function } => (ext, function, Value::Text(text.to_owned())),
            Request::Argument {
                ext,
                function,
                kind,
                shape,
            } => (ext, function, protocol::argument(kind, shape, text)?),
            Request::Shape { .. } | Request::KindOf { .. } => {
                return Err("?shape and ?kind are got, not put".into());
            }
        };
        let key = self.target(ext, function)?;
        self.state().pending.entry(key).or_default().push(value);
        Ok(())
    }
}

impl<S: Store> Store for ExtStore<S> {
    fn get(&self, path: &str) -> Result<String, String> {
        match path.strip_prefix(PREFIX) {
            Some(rest) => self.ext_get(rest).map_err(|e| format!("{path}: {e}")),
            None => match self.facades.get(path) {
                Some(source) => Ok(source.clone()),
                None => self.inner.get(path),
            },
        }
    }

    fn put(&self, path: &str, text: &str) -> Result<(), String> {
        match path.strip_prefix(PREFIX) {
            Some(rest) => self.ext_put(rest, text).map_err(|e| format!("{path}: {e}")),
            None => self.inner.put(path, text),
        }
    }

    fn line(&self) -> Result<String, String> {
        self.inner.line()
    }

    fn show(&self, svg: &str) -> Result<(), String> {
        self.inner.show(svg)
    }

    fn take_shown(&self) -> Vec<String> {
        self.inner.take_shown()
    }
}
