//! linalg: the numerical linear algebra X_eTaL should not reinvent,
//! through nalgebra: solving, inverting, determinants, least squares,
//! symmetric eigenvalues and the singular value decomposition. Matrices
//! cross as X_eTaL arrays (row-major, Float); a vector right-hand side
//! may be a vector or a matrix of columns.

use nalgebra::DMatrix;
use xetal_ext_sdk::{Array, ArrayData, OwnedError, Value, float_vector};

/// Below this a pivot or singular value counts as zero, relative to the
/// largest.
pub const EPS: f64 = 1e-12;

fn invalid(m: impl Into<String>) -> OwnedError {
    OwnedError::invalid_argument(m)
}

fn failure(m: impl Into<String>) -> OwnedError {
    OwnedError::failure(m)
}

/// A matrix argument (a vector is one column when `column` is true).
fn matrix(v: &Value, what: &str, column: bool) -> Result<DMatrix<f64>, OwnedError> {
    let (shape, data) = float_vector(v)?;
    let (m, n) = match shape[..] {
        [m, n] => (m, n),
        [m] if column => (m, 1),
        _ => {
            return Err(invalid(format!(
                "{what} must be a matrix, not shape {shape:?}"
            )));
        }
    };
    if m == 0 || n == 0 {
        return Err(invalid(format!("{what} is empty")));
    }
    if data.iter().any(|x| !x.is_finite()) {
        return Err(invalid(format!("{what} has a NaN or an infinity")));
    }
    Ok(DMatrix::from_row_slice(m, n, &data))
}

fn square(a: &DMatrix<f64>, what: &str) -> Result<(), OwnedError> {
    if a.is_square() {
        Ok(())
    } else {
        Err(invalid(format!(
            "{what} must be square, not {} by {}",
            a.nrows(),
            a.ncols()
        )))
    }
}

/// A matrix result, row-major; one column comes back as a vector when
/// the input was one.
fn result(a: &DMatrix<f64>, vector: bool) -> Result<Value, OwnedError> {
    let data: Vec<f64> = (0..a.nrows())
        .flat_map(|i| (0..a.ncols()).map(move |j| (i, j)))
        .map(|(i, j)| a[(i, j)])
        .collect();
    let shape = if vector {
        vec![a.nrows()]
    } else {
        vec![a.nrows(), a.ncols()]
    };
    Array::new(shape, ArrayData::Float(data))
        .map(Value::Array)
        .map_err(|e| failure(e.to_string()))
}

fn is_vector(v: &Value) -> bool {
    matches!(v, Value::Array(a) if a.shape().len() == 1)
}

/// A solve that refuses a (nearly) singular matrix.
fn lu_solve(a: &DMatrix<f64>, b: &DMatrix<f64>) -> Result<DMatrix<f64>, OwnedError> {
    let scale = a.amax().max(1.0);
    let lu = a.clone().full_piv_lu();
    let small = lu.u().diagonal().iter().any(|d| d.abs() <= EPS * scale);
    match lu.solve(b) {
        Some(x) if !small => Ok(x),
        _ => Err(failure("the matrix is singular (no unique solution)")),
    }
}

/// a solve b: x with a x = b (a square, b a vector or columns).
fn solve(args: &[Value]) -> Result<Value, OwnedError> {
    let a = matrix(&args[0], "the left (a)", false)?;
    square(&a, "the left (a)")?;
    let b = matrix(&args[1], "the right (b)", true)?;
    if b.nrows() != a.nrows() {
        return Err(invalid(format!(
            "a is {} by {} but b has {} rows",
            a.nrows(),
            a.ncols(),
            b.nrows()
        )));
    }
    result(&lu_solve(&a, &b)?, is_vector(&args[1]))
}

/// inverse a: the inverse of a square matrix.
fn inverse(args: &[Value]) -> Result<Value, OwnedError> {
    let a = matrix(&args[0], "the matrix", false)?;
    square(&a, "the matrix")?;
    let id = DMatrix::identity(a.nrows(), a.ncols());
    result(&lu_solve(&a, &id)?, false)
}

/// det a: the determinant.
fn det(args: &[Value]) -> Result<Value, OwnedError> {
    let a = matrix(&args[0], "the matrix", false)?;
    square(&a, "the matrix")?;
    Ok(Value::Float(a.determinant()))
}

/// a lstsq b: the x minimizing |a x - b| (by the SVD; the smallest x
/// when several do).
fn lstsq(args: &[Value]) -> Result<Value, OwnedError> {
    let a = matrix(&args[0], "the left (a)", false)?;
    let b = matrix(&args[1], "the right (b)", true)?;
    if b.nrows() != a.nrows() {
        return Err(invalid(format!(
            "a is {} by {} but b has {} rows",
            a.nrows(),
            a.ncols(),
            b.nrows()
        )));
    }
    let svd = a.svd(true, true);
    let tol = EPS * svd.singular_values.max().max(1.0);
    let x = svd.solve(&b, tol).map_err(|e| failure(e.to_string()))?;
    result(&x, is_vector(&args[1]))
}

fn symmetric(args: &[Value]) -> Result<DMatrix<f64>, OwnedError> {
    let a = matrix(&args[0], "the matrix", false)?;
    square(&a, "the matrix")?;
    let scale = a.amax().max(1.0);
    if (&a - a.transpose()).amax() > 1e-9 * scale {
        return Err(invalid("the matrix must be symmetric"));
    }
    Ok(a)
}

/// The symmetric eigen decomposition, eigenvalues ascending with their
/// vectors.
fn eigen_sorted(a: DMatrix<f64>) -> (Vec<f64>, DMatrix<f64>) {
    let e = a.symmetric_eigen();
    let mut order: Vec<usize> = (0..e.eigenvalues.len()).collect();
    order.sort_by(|&i, &j| e.eigenvalues[i].total_cmp(&e.eigenvalues[j]));
    let values = order.iter().map(|&i| e.eigenvalues[i]).collect();
    let vectors = DMatrix::from_fn(e.eigenvectors.nrows(), order.len(), |r, c| {
        e.eigenvectors[(r, order[c])]
    });
    (values, vectors)
}

/// eigsym a: a symmetric matrix's eigenvalues, ascending.
fn eigsym(args: &[Value]) -> Result<Value, OwnedError> {
    let (values, _) = eigen_sorted(symmetric(args)?);
    Array::new(vec![values.len()], ArrayData::Float(values))
        .map(Value::Array)
        .map_err(|e| failure(e.to_string()))
}

/// eigvecs a: their eigenvectors, one column each, in the same order.
fn eigvecs(args: &[Value]) -> Result<Value, OwnedError> {
    let (_, vectors) = eigen_sorted(symmetric(args)?);
    result(&vectors, false)
}

/// U, the singular values, V.
type Svd = (DMatrix<f64>, Vec<f64>, DMatrix<f64>);

/// The thin SVD, singular values descending: (U, s, V) with a = U
/// diag(s) V'.
fn svd_sorted(v: &Value) -> Result<Svd, OwnedError> {
    let a = matrix(v, "the matrix", false)?;
    let svd = a.svd(true, true);
    let (Some(u), Some(vt)) = (svd.u, svd.v_t) else {
        return Err(failure("the SVD did not converge"));
    };
    let s = svd.singular_values;
    let mut order: Vec<usize> = (0..s.len()).collect();
    order.sort_by(|&i, &j| s[j].total_cmp(&s[i]));
    let u2 = DMatrix::from_fn(u.nrows(), order.len(), |r, c| u[(r, order[c])]);
    let v2 = DMatrix::from_fn(vt.ncols(), order.len(), |r, c| vt[(order[c], r)]);
    Ok((u2, order.iter().map(|&i| s[i]).collect(), v2))
}

/// svd_s a: the singular values, descending.
fn svd_s(args: &[Value]) -> Result<Value, OwnedError> {
    let (_, s, _) = svd_sorted(&args[0])?;
    Array::new(vec![s.len()], ArrayData::Float(s))
        .map(Value::Array)
        .map_err(|e| failure(e.to_string()))
}

/// svd_u a: the left singular vectors, one column each (m by r).
fn svd_u(args: &[Value]) -> Result<Value, OwnedError> {
    result(&svd_sorted(&args[0])?.0, false)
}

/// svd_v a: the right singular vectors, one column each (n by r).
fn svd_v(args: &[Value]) -> Result<Value, OwnedError> {
    result(&svd_sorted(&args[0])?.2, false)
}

xetal_ext_sdk::xetal_extension! {
    name: "linalg",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        solve: 2, "(Num a, Num b) => a -> b -> Float", "a solve b: x with a x = b (a square; b a vector or columns).";
        inverse: 1, "Num a => a -> Float", "The inverse of a square matrix.";
        det: 1, "Num a => a -> Float", "The determinant of a square matrix.";
        lstsq: 2, "(Num a, Num b) => a -> b -> Float", "a lstsq b: the x minimizing |a x - b| (least squares).";
        eigsym: 1, "Num a => a -> Float", "A symmetric matrix's eigenvalues, ascending.";
        eigvecs: 1, "Num a => a -> Float", "A symmetric matrix's eigenvectors, one column each, in that order.";
        svd_s: 1, "Num a => a -> Float", "The singular values, descending.";
        svd_u: 1, "Num a => a -> Float", "The left singular vectors, one column each (m by r).";
        svd_v: 1, "Num a => a -> Float", "The right singular vectors, one column each (n by r).";
    }
}
