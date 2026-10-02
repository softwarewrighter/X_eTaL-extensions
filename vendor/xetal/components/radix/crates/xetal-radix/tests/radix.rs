//! Encode and decode kernels (B12): digits in a mixed radix, and back.

use proptest::prelude::*;
use xetal_radix::{decode, encode};

#[test]
fn encode_gives_digits_most_significant_first() {
    assert_eq!(encode(&[2, 2, 2], 5).unwrap(), vec![1, 0, 1]);
    assert_eq!(encode(&[24, 60, 60], 3725).unwrap(), vec![1, 2, 5]);
    assert_eq!(encode(&[2, 2], 7).unwrap(), vec![1, 1]);
    assert_eq!(encode(&[], 7).unwrap(), Vec::<i64>::new());
}

#[test]
fn a_zero_radix_takes_the_rest() {
    assert_eq!(encode(&[0, 10], 123).unwrap(), vec![12, 3]);
}

#[test]
fn negative_numbers_and_radixes_follow_apl_residue() {
    assert_eq!(encode(&[2, 2, 2], -1).unwrap(), vec![1, 1, 1]);
    assert_eq!(encode(&[-3], 4).unwrap(), vec![-2]);
    assert_eq!(encode(&[10], i64::MIN).unwrap(), vec![2]);
}

#[test]
fn decode_is_horner() {
    assert_eq!(decode(&[2, 2, 2], &[1, 0, 1]).unwrap(), 5);
    assert_eq!(decode(&[24, 60, 60], &[1, 2, 5]).unwrap(), 3725);
    assert_eq!(decode(&[], &[]).unwrap(), 0);
    let nines = [9; 20];
    assert!(decode(&[10; 20], &nines).is_err());
}

proptest! {
    #[test]
    fn decode_inverts_encode_within_range(
        radix in prop::collection::vec(1i64..20, 0..6),
        seed in any::<u64>(),
    ) {
        let span: i64 = radix.iter().product();
        let x = (seed % span as u64) as i64;
        let digits = encode(&radix, x).unwrap();
        prop_assert!(digits.iter().zip(&radix).all(|(d, r)| (0..*r).contains(d)));
        prop_assert_eq!(decode(&radix, &digits).unwrap(), x);
    }

    #[test]
    fn encode_inverts_decode_for_digits_in_range(
        pairs in prop::collection::vec((1i64..20, any::<u64>()), 0..6),
    ) {
        let radix: Vec<i64> = pairs.iter().map(|(r, _)| *r).collect();
        let digits: Vec<i64> = pairs.iter().map(|(r, d)| (d % *r as u64) as i64).collect();
        let x = decode(&radix, &digits).unwrap();
        prop_assert_eq!(encode(&radix, x).unwrap(), digits);
    }
}
