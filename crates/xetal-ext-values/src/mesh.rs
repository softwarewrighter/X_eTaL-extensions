//! One mesh from two arrays: corners and triangles.

use xetal_ext_abi::{ArrayData, Value as Abi};
use xetal_value::Value;

use crate::{Error, Narrow, f32s, to_abi};

/// What the first corner is called in the triangles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Base {
    /// The first corner is 0.
    Zero,
    /// The first corner is 1, as X_eTaL counts.
    One,
}

/// A checked mesh: N corners (`x y z` each, finite) and T triangles
/// (three corners each, counted from 0, every one below N).
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    positions: Vec<f64>,
    indices: Vec<u32>,
}

impl Mesh {
    /// The corners, row-major: `3 * vertices()` numbers.
    #[must_use]
    pub fn positions(&self) -> &[f64] {
        &self.positions
    }

    /// The triangles' corners, counted from 0: `3 * triangles()`.
    #[must_use]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    #[must_use]
    pub fn vertices(&self) -> usize {
        self.positions.len() / 3
    }

    #[must_use]
    pub fn triangles(&self) -> usize {
        self.indices.len() / 3
    }

    /// The corners as `f32`, under a narrowing policy the caller names.
    ///
    /// # Errors
    ///
    /// The first coordinate the policy refuses.
    pub fn positions_f32(&self, caller: &str, policy: Narrow) -> Result<Vec<f32>, Error> {
        f32s(caller, &self.positions, policy)
    }

    /// The mesh's two buffers, given up.
    #[must_use]
    pub fn into_parts(self) -> (Vec<f64>, Vec<u32>) {
        (self.positions, self.indices)
    }
}

/// A mesh from ABI values: `positions` an N by 3 Float array,
/// `triangles` a T by 3 Int array of corners counted from `base`.
/// An empty mesh (no corners, no triangles) is a mesh.
///
/// # Errors
///
/// A value that is not such an array (its kind or shape is said), a
/// coordinate that is not finite, a corner number outside the corners;
/// each names the caller and the place.
pub fn mesh(caller: &str, positions: &Abi, triangles: &Abi, base: Base) -> Result<Mesh, Error> {
    let corners = rows(caller, "positions", positions)?;
    let ArrayData::Float(xyz) = corners else {
        return Err(Error::new(
            caller,
            format!("positions are {}, not Floats", kind(corners)),
        ));
    };
    if let Some(i) = xyz.iter().position(|x| !x.is_finite()) {
        return Err(Error::new(
            caller,
            format!(
                "positions: corner {} has a coordinate that is not finite ({})",
                i / 3 + 1,
                xyz[i]
            ),
        ));
    }
    let n = xyz.len() / 3;
    if u32::try_from(n).is_err() {
        return Err(Error::new(
            caller,
            format!("positions: {n} corners are more than a mesh holds"),
        ));
    }
    let faces = rows(caller, "triangles", triangles)?;
    let ArrayData::Int(ids) = faces else {
        return Err(Error::new(
            caller,
            format!("triangles are {}, not Ints", kind(faces)),
        ));
    };
    let first = i64::from(base == Base::One);
    let mut indices = Vec::with_capacity(ids.len());
    for (i, &id) in ids.iter().enumerate() {
        let at = id
            .checked_sub(first)
            .and_then(|k| usize::try_from(k).ok())
            .filter(|&k| k < n)
            .and_then(|k| u32::try_from(k).ok());
        let Some(at) = at else {
            let range = if n == 0 {
                "there are no corners".to_owned()
            } else {
                format!("the corners are {first} to {}", n as i64 - 1 + first)
            };
            return Err(Error::new(
                caller,
                format!(
                    "triangles: triangle {} names corner {id}; {range}",
                    i / 3 + 1
                ),
            ));
        };
        indices.push(at);
    }
    Ok(Mesh {
        positions: xyz.clone(),
        indices,
    })
}

/// [`mesh`] from the evaluator's values.
///
/// # Errors
///
/// As [`mesh`], and when a value does not cross ([`to_abi`]).
pub fn mesh_of(
    caller: &str,
    positions: &Value<'_>,
    triangles: &Value<'_>,
    base: Base,
) -> Result<Mesh, Error> {
    mesh(
        caller,
        &to_abi(caller, positions)?,
        &to_abi(caller, triangles)?,
        base,
    )
}

/// The data of a K by 3 array.
fn rows<'v>(caller: &str, what: &str, value: &'v Abi) -> Result<&'v ArrayData, Error> {
    let Abi::Array(a) = value else {
        return Err(Error::new(
            caller,
            format!("{what} are not an array of rows of 3"),
        ));
    };
    match a.shape() {
        [_, 3] => Ok(a.data()),
        shape => Err(Error::new(
            caller,
            format!(
                "{what} have shape {}, not rows of 3",
                shape
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        )),
    }
}

fn kind(data: &ArrayData) -> &'static str {
    match data {
        ArrayData::Int(_) => "Ints",
        ArrayData::Float(_) => "Floats",
        ArrayData::Bool(_) => "Bools",
        ArrayData::Char(_) => "Chars",
    }
}
