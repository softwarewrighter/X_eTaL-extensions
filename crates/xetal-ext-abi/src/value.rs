//! Owned values: what hosts and extension authors work with.

use crate::error::{ErrorCode, ValueError};
use crate::model::{MAX_ELEMENTS, MAX_RANK};

/// An X_eTaL value as it crosses the boundary: a scalar, text (a Char
/// vector), or a dense array of any rank.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Array(Array),
}

/// The elements of an array, row-major.
#[derive(Clone, Debug, PartialEq)]
pub enum ArrayData {
    Bool(Vec<bool>),
    Int(Vec<i64>),
    Float(Vec<f64>),
    Char(Vec<char>),
}

impl ArrayData {
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Bool(v) => v.len(),
            Self::Int(v) => v.len(),
            Self::Float(v) => v.len(),
            Self::Char(v) => v.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A dense array: a shape and as many elements as its product.
#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    shape: Vec<usize>,
    data: ArrayData,
}

impl Array {
    /// An array of the given shape.
    ///
    /// # Errors
    ///
    /// When the rank is above 9, the element count is above the V1
    /// bound, or the data does not have the shape's element count.
    pub fn new(shape: Vec<usize>, data: ArrayData) -> Result<Self, ValueError> {
        if shape.len() > MAX_RANK {
            return Err(ValueError::RankTooHigh(shape.len()));
        }
        let expected = element_count(&shape)?;
        if expected != data.len() {
            return Err(ValueError::LengthMismatch {
                expected,
                got: data.len(),
            });
        }
        Ok(Self { shape, data })
    }

    /// A vector (rank 1) of the data.
    ///
    /// # Errors
    ///
    /// When there are more elements than the V1 bound.
    pub fn vector(data: ArrayData) -> Result<Self, ValueError> {
        Self::new(vec![data.len()], data)
    }

    #[must_use]
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    #[must_use]
    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    #[must_use]
    pub fn data(&self) -> &ArrayData {
        &self.data
    }

    #[must_use]
    pub fn into_data(self) -> ArrayData {
        self.data
    }
}

/// The product of a shape, bounded by [`MAX_ELEMENTS`].
pub(crate) fn element_count(shape: &[usize]) -> Result<usize, ValueError> {
    let mut n: usize = 1;
    for &axis in shape {
        n = n.checked_mul(axis).ok_or(ValueError::TooManyElements)?;
    }
    if n > MAX_ELEMENTS {
        return Err(ValueError::TooManyElements);
    }
    Ok(n)
}

/// An error a function returns: a code and a message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnedError {
    code: ErrorCode,
    message: String,
}

impl OwnedError {
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// The arguments were the wrong kind or shape.
    #[must_use]
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidArgument, message)
    }

    /// The function ran and failed.
    #[must_use]
    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ExtensionFailure, message)
    }

    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
