//! Notes for an error inside a train (D47). A train desugars to
//! ordinary application, so an error there is reported at an
//! application the user never wrote; these notes say which train, which
//! element, what the train means at that element (its written-out
//! form), and, from the built-ins' arities, why it may not fit.

mod hint;
mod say;
mod train;

pub use hint::Arity;
pub use train::train_notes;
