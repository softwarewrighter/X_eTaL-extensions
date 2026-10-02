//! Frames shown in turn: each frame's group is visible for its share of
//! a loop (SMIL, so the file animates on its own in any browser).

/// Seconds each frame is shown.
const FRAME_SECONDS: f64 = 0.4;

/// The frames' bodies, animated when there is more than one.
pub fn frames(bodies: &[String]) -> String {
    if let [only] = bodies {
        return only.clone();
    }
    let n = bodies.len();
    let dur = format!("{:.1}s", n as f64 * FRAME_SECONDS);
    bodies
        .iter()
        .enumerate()
        .map(|(k, body)| group(k, n, &dur, body))
        .collect()
}

fn group(k: usize, n: usize, dur: &str, body: &str) -> String {
    let values: Vec<&str> = (0..n).map(|i| if i == k { "1" } else { "0" }).collect();
    format!(
        "<g opacity=\"{}\"><animate attributeName=\"opacity\" values=\"{}\" calcMode=\"discrete\" \
         dur=\"{dur}\" repeatCount=\"indefinite\"/>\n{body}</g>\n",
        values[0],
        values.join(";")
    )
}
