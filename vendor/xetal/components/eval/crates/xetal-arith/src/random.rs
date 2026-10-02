//! A small random generator for `r_oll!` (B7): SplitMix64, seeded from
//! the operating system's per-process hash keys unless a seed is given
//! (a seed is for tests that need the same rolls twice).

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// The generator for `seed`: the same seed gives the same rolls.
    pub fn seeded(seed: u64) -> Self {
        Rng(seed)
    }

    /// A fresh, unpredictable seed.
    pub fn fresh_seed() -> u64 {
        let mut h = RandomState::new().build_hasher();
        h.write_u128(
            std::time::SystemTime::UNIX_EPOCH
                .elapsed()
                .map_or(0, |d| d.as_nanos()),
        );
        h.finish()
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform number in `1..=n` (`n >= 1`), without modulo bias.
    pub fn roll(&mut self, n: u64) -> u64 {
        let zone = u64::MAX - u64::MAX % n;
        loop {
            let x = self.next();
            if x < zone {
                return x % n + 1;
            }
        }
    }
}
