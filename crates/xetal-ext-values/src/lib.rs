//! Typed values between the X_eTaL evaluator and ABI V1, with no text
//! in between.
//!
//! A host that holds an evaluator value (`xetal_value::Value`, of the
//! X_eTaL this repository is pinned to) turns it into an ABI value
//! ([`to_abi`]) to give to an extension, and an extension's reply back
//! ([`from_abi`]). [`mesh`] checks the workload this was built for: an
//! N by 3 Float array of corners and a T by 3 Int array of triangles as
//! one mesh. [`f32s`] narrows Floats to `f32` only when asked, under a
//! named policy. [`Published`] reads values a running program left for
//! its host, through the evaluator's output hook.
//!
//! Every failure names its caller: the text a host gives each call
//! (a function's name, a file and line) begins the message.
//!
//! This is the host's side only. A call from X_eTaL code still crosses
//! as text (`ext:` paths, `docs/bridge.md`) until X_eTaL has a native
//! hook (ask E1); and this crate reads the evaluator's own
//! representation, so it is pinned with the runtime it was built on.

mod convert;
mod error;
mod mesh;
mod narrow;
mod published;

pub use convert::{from_abi, to_abi};
pub use error::Error;
pub use mesh::{Base, Mesh, mesh, mesh_of};
pub use narrow::{Narrow, f32s};
pub use published::Published;
