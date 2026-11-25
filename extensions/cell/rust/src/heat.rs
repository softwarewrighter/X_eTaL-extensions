//! A thermal network: nodes with a heat capacity and a temperature,
//! conductances between them and to the air, and heaters that hold a
//! node at a set point with limited power. Integrated by explicit steps
//! of at most 1 ms, small enough to be stable (under 0.4 of the fastest
//! node's own time constant); a node too small to integrate at that step
//! (a speck of solder) is taken as always in balance with its
//! neighbors. Pure: no clocks, the time is what the caller says.

/// A heater on a node: it holds it at `set` C with at most `max_w` W,
/// `gain` W a kelvin below the set point (a proportional controller).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Heater {
    /// The set point, C.
    pub set: f64,
    /// The most it gives, W.
    pub max_w: f64,
    /// W a kelvin of error.
    pub gain: f64,
}

/// The network.
#[derive(Clone, Debug, Default)]
pub struct Network {
    /// Each node's heat capacity, J/K.
    pub c: Vec<f64>,
    /// Each node's temperature, C.
    pub t: Vec<f64>,
    /// Each node's conductance to the air, W/K.
    pub air: Vec<f64>,
    /// Each node's heater, if any.
    pub heater: Vec<Option<Heater>>,
    /// Conductances between nodes: a, b, W/K.
    pub links: Vec<(usize, usize, f64)>,
    /// The air's temperature, C.
    pub ambient: f64,
}

/// The longest step taken, s.
pub const MAX_STEP: f64 = 0.001;

impl Network {
    /// A node: its heat capacity, temperature and conductance to the air;
    /// its index.
    pub fn node(&mut self, c: f64, t: f64, air: f64) -> usize {
        self.c.push(c);
        self.t.push(t);
        self.air.push(air);
        self.heater.push(None);
        self.c.len() - 1
    }

    /// A conductance between two nodes.
    pub fn link(&mut self, a: usize, b: usize, g: f64) {
        if a != b && g > 0.0 {
            self.links.push((a, b, g));
        }
    }

    /// The step the network is integrated at, and which nodes are taken
    /// as in balance.
    fn plan(&self, total: &[f64]) -> (f64, Vec<bool>) {
        let mut dt = MAX_STEP;
        let mut balanced = vec![false; self.c.len()];
        for i in 0..self.c.len() {
            let g = total[i] + self.air[i] + self.heater[i].map_or(0.0, |h| h.gain);
            if g <= 0.0 {
                continue;
            }
            let tau = self.c[i] / g;
            // a node far faster than a step is in balance with its neighbors
            if tau < MAX_STEP / 20.0 && self.heater[i].is_none() {
                balanced[i] = true;
            } else {
                dt = dt.min(0.4 * tau);
            }
        }
        (dt, balanced)
    }

    /// Runs for `seconds`; `each` is called every `every` seconds (and at
    /// the end) with the time from the start and the temperatures.
    pub fn run(&mut self, seconds: f64, every: f64, mut each: impl FnMut(f64, &[f64])) {
        let n = self.c.len();
        if n == 0 || seconds <= 0.0 {
            return;
        }
        let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
        let mut total = vec![0.0; n];
        for &(a, b, g) in &self.links {
            adj[a].push((b, g));
            adj[b].push((a, g));
            total[a] += g;
            total[b] += g;
        }
        let (dt, balanced) = self.plan(&total);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let steps = (seconds / dt).ceil() as usize;
        #[allow(clippy::cast_precision_loss)]
        let dt = seconds / steps as f64;
        let mut flow = vec![0.0; n];
        let mut next = if every > 0.0 { every } else { f64::INFINITY };
        for s in 1..=steps {
            for i in 0..n {
                if balanced[i] {
                    continue;
                }
                let mut q: f64 = adj[i]
                    .iter()
                    .map(|&(j, g)| g * (self.t[j] - self.t[i]))
                    .sum();
                q += self.air[i] * (self.ambient - self.t[i]);
                if let Some(h) = self.heater[i] {
                    q += (h.gain * (h.set - self.t[i])).clamp(0.0, h.max_w);
                }
                flow[i] = q;
            }
            for i in 0..n {
                if !balanced[i] {
                    self.t[i] += dt * flow[i] / self.c[i];
                }
            }
            // the nodes in balance: the weighted mean of their neighbors and the air
            for i in 0..n {
                if balanced[i] {
                    let (mut w, mut sum) = (self.air[i], self.air[i] * self.ambient);
                    for &(j, g) in &adj[i] {
                        w += g;
                        sum += g * self.t[j];
                    }
                    if w > 0.0 {
                        self.t[i] = sum / w;
                    }
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let now = s as f64 * dt;
            if now + 1e-12 >= next || s == steps {
                each(now, &self.t);
                while next <= now + 1e-12 {
                    next += every;
                }
            }
        }
    }
}
