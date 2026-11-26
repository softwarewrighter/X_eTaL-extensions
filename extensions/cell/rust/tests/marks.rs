//! The decoders (step 22): the resistor color code both ways and from
//! either end, EIA capacitor codes, a kind's marking from its mark.

use xetal_ext_cell::marks::{
    Side, decode_bands, e24, eia_code, eia_pf, marking, nearest_color, parse_ohms, resistor_bands,
};

fn bands(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_string).collect()
}

#[test]
fn resistor_values_parse() {
    for (t, v) in [
        ("4.7k", 4700.0),
        ("4k7", 4700.0),
        ("470", 470.0),
        ("1M", 1e6),
        ("2.2 kilohm", 2200.0),
        ("10R", 10.0),
        ("4R7", 4.7),
    ] {
        assert!((parse_ohms(t).unwrap() - v).abs() < 1e-9, "{t}");
    }
    assert!(parse_ohms("lots").is_err());
}

#[test]
fn bands_both_ways() {
    assert_eq!(
        resistor_bands(4700.0, false).unwrap(),
        bands("yellow violet red gold")
    );
    assert_eq!(
        resistor_bands(10_000.0, false).unwrap(),
        bands("brown black orange gold")
    );
    assert_eq!(
        resistor_bands(4.7, false).unwrap(),
        bands("yellow violet gold gold")
    );
    assert_eq!(
        resistor_bands(0.47, false).unwrap(),
        bands("yellow violet silver gold")
    );
    assert_eq!(
        resistor_bands(12_100.0, true).unwrap(),
        bands("brown red brown red brown")
    );
    for b in [
        "yellow violet red gold",
        "brown black orange gold",
        "brown red brown red brown",
    ] {
        let (v, _) = decode_bands(&bands(b)).unwrap();
        let five = b.split_whitespace().count() == 5;
        assert_eq!(resistor_bands(v, five).unwrap(), bands(b));
    }
    assert_eq!(
        decode_bands(&bands("yellow violet red gold")).unwrap(),
        (4700.0, 5.0)
    );
}

#[test]
fn a_resistor_read_from_its_other_end() {
    // gold first: it was read backward
    assert_eq!(
        decode_bands(&bands("gold red violet yellow")).unwrap(),
        (4700.0, 5.0)
    );
    // nothing sensible either way
    assert!(decode_bands(&bands("white white white white")).is_err());
    assert!(decode_bands(&bands("red red")).is_err());
}

#[test]
fn e24_values() {
    assert!(e24(4700.0) && e24(10.0) && e24(9.1e5));
    assert!(!e24(4800.0));
}

#[test]
fn eia_codes_both_ways() {
    for (c, pf) in [
        ("104", 100_000.0),
        ("103", 10_000.0),
        ("472", 4700.0),
        ("4R7", 4.7),
        ("47", 47.0),
        ("0.1", 100_000.0),
    ] {
        assert!((eia_pf(c).unwrap() - pf).abs() < 1e-6, "{c}");
    }
    assert_eq!(eia_code(100_000.0), "104");
    assert_eq!(eia_code(4700.0), "472");
    assert_eq!(eia_code(4.7), "4R7");
    assert_eq!(eia_code(47.0), "47");
    assert!(eia_pf("1047").is_err() && eia_pf("abc").is_err());
}

#[test]
fn markings_of_kinds() {
    let r = marking("axial-resistor", "4.7k").unwrap();
    assert_eq!(r.bands, bands("yellow violet red gold"));
    assert_eq!(r.value, "4.7 kilohm, 5%");
    assert_eq!(r.side, Side::Around);
    let c = marking("ceramic-disc", "104").unwrap();
    assert_eq!((c.value.as_str(), c.side), ("100 nF", Side::FacePlusY));
    let u = marking("dip8", "NE555P").unwrap();
    assert!(u.pin1_dot && u.side == Side::Top && u.text == ["NE555P"]);
    let e = marking("radial-electrolytic", "10uF 25V").unwrap();
    assert_eq!(e.text, ["10uF", "25V"]);
    assert_eq!(e.value, "10 uF, 25 V");
    assert_eq!(
        marking("led-5mm", "red").unwrap().color.as_deref(),
        Some("red")
    );
    assert!(marking("led-5mm", "plaid").is_err());
    assert!(marking("battery-snap", "x").is_err());
    // three significant digits: five bands
    assert_eq!(marking("axial-resistor", "12.1k").unwrap().bands.len(), 5);
}

#[test]
fn colors_seen_in_a_picture() {
    assert_eq!(nearest_color([200.0, 30.0, 30.0]), ("red", 1.0));
    let (c, q) = nearest_color([215.0, 90.0, 25.0]);
    assert!(c == "orange" || c == "red", "{c}");
    assert!(q < 0.9);
}

#[test]
fn bands_read_from_a_picture() {
    use xetal_ext_cell::marks::read_bands;
    // as the page sampled them: the body balances the light
    let r = read_bands(
        &[
            [217.0, 201.0, 44.0],
            [127.0, 59.0, 178.0],
            [182.0, 36.0, 37.0],
            [184.0, 151.0, 51.0],
        ],
        [210.0, 201.0, 180.0],
        [237.0, 223.0, 196.0],
    );
    assert_eq!(r.bands, bands("yellow violet red gold"));
    assert_eq!(r.value.as_deref(), Ok("4.7 kilohm, 5%"));
    assert!(r.confidence > 0.6, "{}", r.confidence);
    // the same in a dim bluish light: the body's color corrects it
    let dim = |c: [f64; 3]| [c[0] * 0.5, c[1] * 0.55, c[2] * 0.7];
    let r2 = read_bands(
        &[
            dim([217.0, 201.0, 44.0]),
            dim([127.0, 59.0, 178.0]),
            dim([182.0, 36.0, 37.0]),
            dim([184.0, 151.0, 51.0]),
        ],
        dim([210.0, 201.0, 180.0]),
        [237.0, 223.0, 196.0],
    );
    assert_eq!(r2.bands, r.bands);
    // a blurred picture: every band the body's color -- no value, not sure
    let blur = read_bands(
        &[[210.0, 201.0, 180.0]; 4],
        [210.0, 201.0, 180.0],
        [237.0, 223.0, 196.0],
    );
    assert!(blur.value.is_err());
}
