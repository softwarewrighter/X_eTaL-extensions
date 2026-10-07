//! Small cases with known answers, through the loader.

use xetal_ext_loader::{Array, ArrayData, Registry, Value};

fn la() -> Registry {
    let mut r = Registry::new();
    r.load_static(xetal_ext_linalg::__xetal_extension::descriptor)
        .unwrap();
    r
}

fn m(shape: &[usize], data: &[f64]) -> Value {
    Value::Array(Array::new(shape.to_vec(), ArrayData::Float(data.to_vec())).unwrap())
}

fn unpack(v: &Value) -> (Vec<usize>, Vec<f64>) {
    match v {
        Value::Array(a) => match a.data() {
            ArrayData::Float(x) => (a.shape().to_vec(), x.clone()),
            d => panic!("{d:?}"),
        },
        Value::Float(x) => (vec![], vec![*x]),
        v => panic!("{v:?}"),
    }
}

fn close(a: &[f64], b: &[f64]) {
    assert_eq!(a.len(), b.len(), "{a:?} vs {b:?}");
    for (x, y) in a.iter().zip(b) {
        assert!((x - y).abs() < 1e-9, "{a:?} vs {b:?}");
    }
}

/// Row-major product of an m by k and a k by n matrix.
fn mul(a: &[f64], b: &[f64], m: usize, k: usize, n: usize) -> Vec<f64> {
    (0..m * n)
        .map(|ij| {
            (0..k)
                .map(|t| a[(ij / n) * k + t] * b[t * n + ij % n])
                .sum()
        })
        .collect()
}

#[test]
fn solve_inverse_det() {
    let r = la();
    let a = m(&[2, 2], &[2.0, 1.0, 1.0, 3.0]);
    let x = r
        .call("linalg", "solve", &[a.clone(), m(&[2], &[3.0, 5.0])])
        .unwrap();
    assert_eq!(unpack(&x).0, [2]);
    close(&unpack(&x).1, &[0.8, 1.4]);
    // columns on the right
    let x = r
        .call(
            "linalg",
            "solve",
            &[a.clone(), m(&[2, 2], &[3.0, 1.0, 5.0, 0.0])],
        )
        .unwrap();
    assert_eq!(unpack(&x).0, [2, 2]);
    close(&unpack(&x).1, &[0.8, 0.6, 1.4, -0.2]);
    let inv = r
        .call("linalg", "inverse", std::slice::from_ref(&a))
        .unwrap();
    close(&unpack(&inv).1, &[0.6, -0.2, -0.2, 0.4]);
    let d = r
        .call(
            "linalg",
            "det",
            &[m(&[3, 3], &[2.0, 0.0, 1.0, 1.0, 3.0, 2.0, 1.0, 1.0, 2.0])],
        )
        .unwrap();
    close(&unpack(&d).1, &[6.0]);
    // singular, not square, mismatched
    let s = m(&[2, 2], &[1.0, 2.0, 2.0, 4.0]);
    let e = r
        .call("linalg", "solve", &[s.clone(), m(&[2], &[1.0, 2.0])])
        .unwrap_err();
    assert!(e.to_string().contains("singular"), "{e}");
    assert!(
        r.call("linalg", "inverse", &[s])
            .unwrap_err()
            .to_string()
            .contains("singular")
    );
    let e = r
        .call("linalg", "det", &[m(&[2, 3], &[0.0; 6])])
        .unwrap_err();
    assert!(e.to_string().contains("square"), "{e}");
    let e = r
        .call("linalg", "solve", &[a, m(&[3], &[1.0; 3])])
        .unwrap_err();
    assert!(e.to_string().contains("rows"), "{e}");
}

#[test]
fn least_squares_fits_a_line() {
    // y = 1 + 2 x exactly, at x = 0 1 2 3
    let a = m(&[4, 2], &[1.0, 0.0, 1.0, 1.0, 1.0, 2.0, 1.0, 3.0]);
    let x = la()
        .call("linalg", "lstsq", &[a, m(&[4], &[1.0, 3.0, 5.0, 7.0])])
        .unwrap();
    close(&unpack(&x).1, &[1.0, 2.0]);
}

#[test]
fn symmetric_eigen() {
    let r = la();
    let a = m(&[2, 2], &[2.0, 1.0, 1.0, 2.0]);
    close(
        &unpack(
            &r.call("linalg", "eigsym", std::slice::from_ref(&a))
                .unwrap(),
        )
        .1,
        &[1.0, 3.0],
    );
    let (shape, v) = unpack(&r.call("linalg", "eigvecs", &[a]).unwrap());
    assert_eq!(shape, [2, 2]);
    // the columns: along (1, -1) and (1, 1), up to sign
    let h = 0.5_f64.sqrt();
    close(
        &[v[0].abs(), v[2].abs(), v[1].abs(), v[3].abs()],
        &[h, h, h, h],
    );
    assert!(v[0] * v[2] < 0.0 && v[1] * v[3] > 0.0);
    let e = r
        .call("linalg", "eigsym", &[m(&[2, 2], &[1.0, 2.0, 3.0, 4.0])])
        .unwrap_err();
    assert!(e.to_string().contains("symmetric"), "{e}");
}

#[test]
fn svd_rebuilds_the_matrix() {
    let r = la();
    let data = [3.0, 2.0, 2.0, 2.0, 3.0, -2.0];
    let a = m(&[2, 3], &data);
    let (_, s) = unpack(&r.call("linalg", "svd_s", std::slice::from_ref(&a)).unwrap());
    close(&s, &[5.0, 3.0]);
    let (us, u) = unpack(&r.call("linalg", "svd_u", std::slice::from_ref(&a)).unwrap());
    let (vs, v) = unpack(&r.call("linalg", "svd_v", &[a]).unwrap());
    assert_eq!((us, vs), (vec![2, 2], vec![3, 2]));
    // U diag(s) V' is the matrix again
    let us_: Vec<f64> = (0..4).map(|i| u[i] * s[i % 2]).collect();
    let vt: Vec<f64> = (0..6).map(|i| v[(i % 3) * 2 + i / 3]).collect();
    close(&mul(&us_, &vt, 2, 2, 3), &data);
    let e = r
        .call("linalg", "svd_s", &[m(&[2, 2], &[1.0, f64::NAN, 0.0, 1.0])])
        .unwrap_err();
    assert!(e.to_string().contains("NaN"), "{e}");
}
