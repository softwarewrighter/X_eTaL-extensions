//! The crate against the real evaluator: values a program worked out,
//! across to ABI values and back, with no text in between.

use xetal_arith::Rng;
use xetal_ext_abi::{Array, ArrayData, Value as Abi};
use xetal_ext_values::{Base, Narrow, Published, f32s, from_abi, mesh, mesh_of, to_abi};
use xetal_value::Value;

/// Run `src` and give its top-level values to `then`.
fn run<R>(src: &str, then: impl FnOnce(&[Value<'_>], &Published<'_>) -> R) -> R {
    let program = xetal_core::lower(src).expect("the program lowers");
    go(&program, then)
}

fn go<'a, R>(
    program: &'a xetal_core::Program,
    then: impl FnOnce(&[Value<'a>], &Published<'a>) -> R,
) -> R {
    let published = Published::new();
    let mut seen: Vec<Value<'a>> = Vec::new();
    {
        let mut hook = published.hook();
        let mut keep = |v: &Value<'a>| {
            hook(v);
            seen.push(v.clone());
        };
        let mut out = Vec::new();
        let mut machine = xetal_step::Machine::new(program, &mut out, Rng::seeded(1))
            .hooks(Some(&mut keep), None);
        machine.finish().expect("the program runs");
    }
    then(&seen, &published)
}

fn floats(shape: &[usize], data: &[f64]) -> Abi {
    Abi::Array(Array::new(shape.to_vec(), ArrayData::Float(data.to_vec())).unwrap())
}

fn ints(shape: &[usize], data: &[i64]) -> Abi {
    Abi::Array(Array::new(shape.to_vec(), ArrayData::Int(data.to_vec())).unwrap())
}

const SQUARE: &str =
    "4 3 r_eshape 0.0 0.0 0.0 1.0 0.0 0.0 1.0 0.0 1.0 0.0 0.0 1.0\n2 3 r_eshape 1 2 3 1 3 4\n";

#[test]
fn the_forks_workload_is_one_mesh() {
    run(SQUARE, |v, _| {
        let m = mesh_of("terrain", &v[0], &v[1], Base::One).unwrap();
        assert_eq!((m.vertices(), m.triangles()), (4, 2));
        assert_eq!(m.indices(), [0, 1, 2, 0, 2, 3]);
        assert_eq!(m.positions()[3..6], [1.0, 0.0, 0.0]);
        assert_eq!(m.positions_f32("terrain", Narrow::Exact).unwrap().len(), 12);
    });
}

#[test]
fn arrays_cross_typed_and_come_back_the_same() {
    run(SQUARE, |v, _| {
        let a = to_abi("t", &v[0]).unwrap();
        assert_eq!(
            a,
            floats(
                &[4, 3],
                &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0]
            )
        );
        let b = to_abi("t", &v[1]).unwrap();
        assert_eq!(b, ints(&[2, 3], &[1, 2, 3, 1, 3, 4]));
        for abi in [a, b] {
            let back: Value<'_> = from_abi("t", &abi).unwrap();
            assert_eq!(to_abi("t", &back).unwrap(), abi);
        }
    });
}

#[test]
fn scalars_text_bools_and_empties_cross() {
    run(
        "7\n2.5\n\"abc\"\n1 2 3 = 1 5 3\n0 3 r_eshape 0.0\n\"\"\n2 2 r_eshape \"abcd\"\n",
        |v, _| {
            assert_eq!(to_abi("t", &v[0]).unwrap(), Abi::Int(7));
            assert_eq!(to_abi("t", &v[1]).unwrap(), Abi::Float(2.5));
            assert_eq!(to_abi("t", &v[2]).unwrap(), Abi::Text("abc".into()));
            assert_eq!(
                to_abi("t", &v[3]).unwrap(),
                Abi::Array(Array::vector(ArrayData::Bool(vec![true, false, true])).unwrap())
            );
            assert_eq!(to_abi("t", &v[4]).unwrap(), floats(&[0, 3], &[]));
            assert_eq!(to_abi("t", &v[5]).unwrap(), Abi::Text(String::new()));
            let grid = to_abi("t", &v[6]).unwrap();
            assert_eq!(
                grid,
                Abi::Array(
                    Array::new(vec![2, 2], ArrayData::Char("abcd".chars().collect())).unwrap()
                )
            );
            for (i, value) in v.iter().enumerate() {
                let abi = to_abi("t", value).unwrap();
                let back: Value<'_> = from_abi("t", &abi).unwrap();
                assert_eq!(to_abi("t", &back).unwrap(), abi, "value {i}");
            }
        },
    );
}

#[test]
fn what_the_abi_does_not_carry_is_refused_by_name() {
    run("{ x -> x }\n(1 2 3 , \"a\")\n", |v, _| {
        let e = to_abi("Game.xtl draw", &v[0]).unwrap_err();
        assert_eq!(
            e.to_string(),
            "Game.xtl draw: a function does not cross the extension boundary"
        );
        assert_eq!(e.caller(), "Game.xtl draw");
        let e = to_abi("Game.xtl draw", &v[1]).unwrap_err();
        assert!(e.message().contains("does not cross"), "{e}");
    });
}

#[test]
fn malformed_meshes_name_the_caller_and_the_place() {
    let good = floats(&[2, 3], &[0.0; 6]);
    let tri = ints(&[1, 3], &[1, 2, 2]);
    let says = |p: &Abi, t: &Abi, base| mesh("snow/chunk", p, t, base).unwrap_err().to_string();
    assert_eq!(
        says(&floats(&[3, 2], &[0.0; 6]), &tri, Base::One),
        "snow/chunk: positions have shape 3 2, not rows of 3"
    );
    assert_eq!(
        says(&floats(&[6], &[0.0; 6]), &tri, Base::One),
        "snow/chunk: positions have shape 6, not rows of 3"
    );
    assert_eq!(
        says(&Abi::Float(1.0), &tri, Base::One),
        "snow/chunk: positions are not an array of rows of 3"
    );
    assert_eq!(
        says(&ints(&[2, 3], &[0; 6]), &tri, Base::One),
        "snow/chunk: positions are Ints, not Floats"
    );
    assert_eq!(
        says(&good, &floats(&[1, 3], &[1.0, 2.0, 2.0]), Base::One),
        "snow/chunk: triangles are Floats, not Ints"
    );
    assert_eq!(
        says(&good, &ints(&[3], &[1, 2, 2]), Base::One),
        "snow/chunk: triangles have shape 3, not rows of 3"
    );
    assert_eq!(
        says(
            &floats(&[2, 3], &[0.0, 0.0, 0.0, 0.0, f64::NAN, 0.0]),
            &tri,
            Base::One
        ),
        "snow/chunk: positions: corner 2 has a coordinate that is not finite (NaN)"
    );
    assert_eq!(
        says(
            &floats(&[2, 3], &[f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0]),
            &tri,
            Base::One
        ),
        "snow/chunk: positions: corner 1 has a coordinate that is not finite (inf)"
    );
    assert_eq!(
        says(&good, &ints(&[2, 3], &[1, 2, 2, 1, 3, 2]), Base::One),
        "snow/chunk: triangles: triangle 2 names corner 3; the corners are 1 to 2"
    );
    assert_eq!(
        says(&good, &ints(&[1, 3], &[0, 1, 2]), Base::One),
        "snow/chunk: triangles: triangle 1 names corner 0; the corners are 1 to 2"
    );
    assert_eq!(
        says(&good, &ints(&[1, 3], &[0, 1, 2]), Base::Zero),
        "snow/chunk: triangles: triangle 1 names corner 2; the corners are 0 to 1"
    );
    assert_eq!(
        says(&good, &ints(&[1, 3], &[-1, 1, 1]), Base::Zero),
        "snow/chunk: triangles: triangle 1 names corner -1; the corners are 0 to 1"
    );
    assert_eq!(
        says(&good, &ints(&[1, 3], &[i64::MIN, 1, 1]), Base::One),
        format!(
            "snow/chunk: triangles: triangle 1 names corner {}; the corners are 1 to 2",
            i64::MIN
        )
    );
    assert_eq!(
        says(&floats(&[0, 3], &[]), &tri, Base::One),
        "snow/chunk: triangles: triangle 1 names corner 1; there are no corners"
    );
    let empty = mesh(
        "snow/chunk",
        &floats(&[0, 3], &[]),
        &ints(&[0, 3], &[]),
        Base::One,
    )
    .unwrap();
    assert_eq!((empty.vertices(), empty.triangles()), (0, 0));
}

#[test]
fn narrowing_is_asked_for_and_checked() {
    assert_eq!(
        f32s("t", &[0.0, 1.5, -2.25], Narrow::Exact).unwrap(),
        [0.0, 1.5, -2.25]
    );
    assert_eq!(f32s("t", &[0.1], Narrow::Rounded).unwrap(), [0.1_f32]);
    let says = |x: f64, policy| f32s("upload", &[1.0, x], policy).unwrap_err().to_string();
    assert_eq!(
        says(0.1, Narrow::Exact),
        "upload: number 2 (0.1) is not exactly an f32"
    );
    assert_eq!(
        says(1e39, Narrow::Rounded),
        "upload: number 2 (1000000000000000000000000000000000000000) is beyond f32's range"
    );
    assert_eq!(
        says(1e-60, Narrow::Rounded),
        "upload: number 2 (0.000000000000000000000000000000000000000000000000000000000001) would round to zero as an f32"
    );
    assert_eq!(
        says(f64::NAN, Narrow::Rounded),
        "upload: number 2 (NaN) is not finite"
    );
    // an f32's own small numbers (subnormals) are kept
    assert_eq!(f32s("t", &[1e-40], Narrow::Rounded).unwrap().len(), 1);
}

#[test]
fn a_host_reads_what_a_program_published() {
    let src = "frame! := 2 3 r_eshape 1.5\nu:s_tep := { k -> frame! := frame! * k }\nq := u:s_tep 2.0\nu:s_tep\n";
    run(src, |_, published| {
        let frame = published.named("host", "frame!").unwrap();
        assert_eq!(to_abi("host", &frame).unwrap(), floats(&[2, 3], &[3.0; 6]));
        assert!(matches!(published.last(), Some(Value::Closure(_))));
        assert_eq!(
            published.named("host", "missing").unwrap_err().to_string(),
            "host: missing is not a variable the program published"
        );
    });
    run("1 + 1\n", |_, published| {
        assert_eq!(
            published.named("host", "frame!").unwrap_err().to_string(),
            "host: the program has published no function yet"
        );
        assert!(matches!(published.last(), Some(Value::Int(2))));
    });
}
