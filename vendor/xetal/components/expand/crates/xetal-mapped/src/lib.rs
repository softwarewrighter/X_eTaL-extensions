//! A text with a map back to the text it was made from: what macro
//! expansion builds, so every byte of an expanded program can be traced
//! to where it was written (a macro's argument, or the call itself).

mod build;
mod map;

pub use map::{Mapped, Piece};
