//! Several source files as one combined program text with a source
//! map (MC1): later stages work on byte offsets into the combined text
//! as for a single file, and every offset maps back to the file, line
//! and column where it was written. A piece of the combined text is
//! either a copy of a file's bytes or a replacement for one of its
//! tokens (a rewritten namespace), which maps to the original token.

mod locate;
mod report;
mod sources;

pub use locate::Location;
pub use sources::Sources;
