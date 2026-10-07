//! Pictures made here (never downloaded), written, read back, resized,
//! through the loader as a host calls them.
#![allow(unsafe_code)]

use std::sync::OnceLock;

use xetal_ext_loader::{Array, ArrayData, Registry, Value};

/// One temporary root for the whole binary, set before any call.
fn im() -> Registry {
    static ROOT: OnceLock<tempfile::TempDir> = OnceLock::new();
    let dir = ROOT.get_or_init(|| tempfile::tempdir().unwrap());
    // SAFETY: set once, to the same value, before any call reads it.
    unsafe { std::env::set_var("XETAL_IMAGE_ROOT", dir.path()) };
    let mut r = Registry::new();
    r.load_static(xetal_ext_image::__xetal_extension::descriptor)
        .unwrap();
    r
}

fn t(s: &str) -> Value {
    Value::Text(s.into())
}

fn floats(shape: Vec<usize>, data: Vec<f64>) -> Value {
    Value::Array(Array::new(shape, ArrayData::Float(data)).unwrap())
}

fn unpack(v: &Value) -> (Vec<usize>, Vec<f64>) {
    match v {
        Value::Array(a) => match a.data() {
            ArrayData::Float(x) => (a.shape().to_vec(), x.clone()),
            ArrayData::Int(i) => (a.shape().to_vec(), i.iter().map(|&i| i as f64).collect()),
            d => panic!("{d:?}"),
        },
        v => panic!("{v:?}"),
    }
}

/// A 3 by 4 gray ramp and a 2 by 2 color picture, in steps of 1/255.
fn ramp() -> Value {
    floats(
        vec![3, 4],
        (0..12).map(|i| f64::from(i * 20) / 255.0).collect(),
    )
}

fn color() -> Value {
    #[rustfmt::skip]
    let px = [255.0, 0.0, 0.0,   0.0, 255.0, 0.0,
              0.0, 0.0, 255.0,   255.0, 255.0, 255.0];
    floats(vec![2, 2, 3], px.iter().map(|v| v / 255.0).collect())
}

#[test]
fn gray_and_color_round_trip_exactly() {
    let r = im();
    let call = |f: &str, a: &[Value]| r.call("image", f, a);
    assert_eq!(
        call("write", &[t("out/ramp.png"), ramp()]).unwrap(),
        Value::Int(12)
    );
    assert_eq!(
        unpack(&call("read", &[t("out/ramp.png")]).unwrap()),
        unpack(&ramp())
    );
    assert_eq!(
        unpack(&call("size", &[t("out/ramp.png")]).unwrap()),
        (vec![2], vec![3.0, 4.0])
    );

    assert_eq!(
        call("write", &[t("out/c.png"), color()]).unwrap(),
        Value::Int(4)
    );
    let (shape, px) = unpack(&call("read", &[t("out/c.png")]).unwrap());
    assert_eq!(shape, [2, 2, 3]);
    assert_eq!(px, unpack(&color()).1);

    // gray: the luminance; red is darker than green, white is 1
    let (shape, g) = unpack(&call("gray", &[t("out/c.png")]).unwrap());
    assert_eq!(shape, [2, 2]);
    assert!(g[0] < g[1] && g[2] < g[0], "{g:?}");
    assert!((g[3] - 1.0).abs() < 1e-6);

    // values outside 0..1 are clamped, NaN is black
    let wild = floats(vec![1, 3], vec![-2.0, 7.0, f64::NAN]);
    call("write", &[t("out/w.png"), wild]).unwrap();
    assert_eq!(
        unpack(&call("read", &[t("out/w.png")]).unwrap()).1,
        [0.0, 1.0, 0.0]
    );

    // JPEG by the name: close, not exact
    call("write", &[t("out/c.jpg"), color()]).unwrap();
    assert_eq!(
        unpack(&call("size", &[t("out/c.jpg")]).unwrap()).1,
        [2.0, 2.0]
    );
}

#[test]
fn resize_keeps_gray_or_color() {
    let r = im();
    let call = |f: &str, a: &[Value]| r.call("image", f, a);
    let flat = floats(vec![4, 4], vec![0.5; 16]);
    let (shape, px) = unpack(&call("resize", &[floats(vec![2], vec![2.0, 8.0]), flat]).unwrap());
    assert_eq!(shape, [2, 8]);
    assert!(
        px.iter().all(|v| (v - 128.0 / 255.0).abs() < 0.01),
        "{px:?}"
    );
    let (shape, _) = unpack(&call("resize", &[floats(vec![2], vec![4.0, 4.0]), color()]).unwrap());
    assert_eq!(shape, [4, 4, 3]);
}

#[test]
fn errors_name_the_problem() {
    let r = im();
    let msg = |f: &str, a: &[Value]| r.call("image", f, a).unwrap_err().to_string();
    assert!(msg("read", &[t("../x.png")]).contains("no .."));
    assert!(msg("read", &[t("/etc/hosts")]).contains("no .."));
    assert!(msg("read", &[t("missing.png")]).contains("missing.png"));
    let bad = floats(vec![2, 2, 2], vec![0.0; 8]);
    assert!(msg("write", &[t("x.png"), bad]).contains("h by w by 3"));
    let v = floats(vec![3], vec![0.0; 3]);
    assert!(msg("write", &[t("x.png"), v]).contains("h by w"));
    assert!(msg("resize", &[floats(vec![1], vec![2.0]), ramp()]).contains("height and width"));
    assert!(msg("resize", &[floats(vec![2], vec![0.0, 2.0]), ramp()]).contains("not a size"));
}
