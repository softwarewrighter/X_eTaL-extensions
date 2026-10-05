//! Arrays as pixels, headless.

use xetal_ext_canvas::{Frame, frame, scaled};
use xetal_ext_sdk::{Array, ArrayData, Value};

fn arr(shape: &[usize], data: ArrayData) -> Value {
    Value::Array(Array::new(shape.to_vec(), data).unwrap())
}

#[test]
fn grey_and_colour() {
    let f = frame(&arr(&[1, 3], ArrayData::Float(vec![0.0, 0.5, 2.0]))).unwrap();
    assert_eq!((f.width, f.height), (3, 1));
    assert_eq!(f.pixels, [0x00_0000, 0x80_8080, 0xff_ffff]);
    let f = frame(&arr(&[1, 2], ArrayData::Bool(vec![true, false]))).unwrap();
    assert_eq!(f.pixels, [0xff_ffff, 0]);
    let f = frame(&arr(&[1, 1, 3], ArrayData::Int(vec![255, 128, 300]))).unwrap();
    assert_eq!(f.pixels, [0xff_80ff]);
    assert!(frame(&arr(&[2, 2, 2], ArrayData::Int(vec![0; 8]))).is_err());
    assert!(frame(&Value::Int(3)).is_err());
    assert!(frame(&arr(&[1, 1], ArrayData::Char(vec!['x']))).is_err());
}

#[test]
fn scaling_keeps_the_shape() {
    let f = Frame {
        width: 2,
        height: 1,
        pixels: vec![1, 2],
    };
    // 4 by 4 window: 2 by 1 scales by 2 to 4 by 2, centered vertically
    let out = scaled(&f, 4, 4);
    let bg = 0x0010_1214;
    assert_eq!(
        out,
        [bg, bg, bg, bg, 1, 1, 2, 2, 1, 1, 2, 2, bg, bg, bg, bg]
    );
    assert_eq!(
        scaled(
            &Frame {
                width: 0,
                height: 0,
                pixels: vec![]
            },
            2,
            1
        ),
        [bg, bg]
    );
}
