//! Array errors; they convert into `xetal_base::Diagnostic`.

use xetal_base::Diagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayError {
    /// The data does not fill the shape.
    Length { shape: Vec<usize>, len: usize },
    /// Two arrays combined element by element have different shapes.
    Shape { left: Vec<usize>, right: Vec<usize> },
    /// An empty array where an item is needed (no fill exists, B10).
    Empty,
    /// A 1-origin index outside `1..=len`.
    Index { index: i64, len: usize },
    /// More items than an array may hold.
    TooLarge,
}

/// The most items one array may hold.
pub const MAX_ITEMS: usize = 1 << 28;

/// The number of items of `shape`, if an array may be that large.
pub fn size(shape: &[usize]) -> Result<usize, ArrayError> {
    shape
        .iter()
        .try_fold(1usize, |n, d| n.checked_mul(*d))
        .filter(|n| *n <= MAX_ITEMS)
        .ok_or(ArrayError::TooLarge)
}

fn dims(shape: &[usize]) -> String {
    let parts: Vec<String> = shape.iter().map(ToString::to_string).collect();
    parts.join(" ")
}

impl From<ArrayError> for Diagnostic {
    fn from(err: ArrayError) -> Self {
        match err {
            ArrayError::Length { shape, len } => Diagnostic::new(
                "length-mismatch",
                format!("{len} items do not fill shape {}", dims(&shape)),
            ),
            ArrayError::Shape { left, right } => Diagnostic::new(
                "shape-mismatch",
                format!("shapes differ: {} and {}", dims(&left), dims(&right)),
            ),
            ArrayError::Empty => Diagnostic::new(
                "empty",
                "the array is empty: there is no item to use (and no fill)",
            ),
            ArrayError::Index { index, len } => {
                Diagnostic::new("index", format!("index {index} is outside 1..{len}"))
            }
            ArrayError::TooLarge => Diagnostic::new(
                "too-large",
                format!("an array may hold at most {MAX_ITEMS} items"),
            ),
        }
    }
}
