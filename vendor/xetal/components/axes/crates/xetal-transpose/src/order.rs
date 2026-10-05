//! Reading a permutation of the axes.

use xetal_base::Diagnostic;

/// The 1-origin axes `p` as 0-origin positions, when they list each of
/// the `rank` axes once.
pub fn permutation(p: &[i64], rank: usize) -> Result<Vec<usize>, Diagnostic> {
    if p.len() != rank {
        return Err(Diagnostic::new(
            "length",
            format!(
                "a permutation of {rank} axes needs {rank} numbers, got {}",
                p.len()
            ),
        ));
    }
    let to: Vec<usize> = p
        .iter()
        .filter_map(|&k| usize::try_from(k).ok().filter(|&k| (1..=rank).contains(&k)))
        .map(|k| k - 1)
        .collect();
    let once = (0..rank).all(|axis| to.contains(&axis));
    match to.len() == rank && once {
        true => Ok(to),
        false => Err(Diagnostic::new(
            "domain",
            format!("a permutation lists each axis once, got {}", spelled(p)),
        )),
    }
}

fn spelled(p: &[i64]) -> String {
    p.iter().map(i64::to_string).collect::<Vec<_>>().join(" ")
}
