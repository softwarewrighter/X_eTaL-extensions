//! The thermal network against what is known in closed form: a lumped
//! RC decay, two nodes exchanging heat, a heater's steady state, and a
//! node too small to integrate in balance with its neighbors.

use xetal_ext_cell::heat::{Heater, Network};

fn net() -> Network {
    Network {
        ambient: 25.0,
        ..Network::default()
    }
}

#[test]
fn a_lumped_node_decays_as_rc() {
    // C = 2 J/K, G = 0.1 W/K to the air: tau = 20 s
    let mut n = net();
    n.node(2.0, 100.0, 0.1);
    let mut at = Vec::new();
    n.run(40.0, 20.0, |t, temps| at.push((t, temps[0])));
    for (t, got) in at {
        let want = 25.0 + 75.0 * (-t / 20.0f64).exp();
        assert!(
            (got - want).abs() / (want - 25.0) < 1e-3,
            "at {t} s: {got} against {want}"
        );
    }
}

#[test]
fn two_nodes_meet_at_their_mean() {
    // equal C, one conductance, no air: the difference decays at 2G/C
    let mut n = net();
    let a = n.node(1.0, 200.0, 0.0);
    let b = n.node(1.0, 40.0, 0.0);
    n.link(a, b, 0.5);
    n.run(1.0, 0.0, |_, _| {});
    let diff = n.t[a] - n.t[b];
    let want = 160.0 * (-1.0f64).exp();
    assert!((diff - want).abs() / want < 1e-3, "{diff} against {want}");
    assert!((n.t[a] + n.t[b] - 240.0).abs() < 1e-9, "energy kept");
}

#[test]
fn a_heater_holds_its_node_near_the_set_point() {
    // proportional: gain (set - T) = air (T - ambient) at rest
    let mut n = net();
    let tip = n.node(0.4, 25.0, 0.01);
    n.heater[tip] = Some(Heater {
        set: 350.0,
        max_w: 60.0,
        gain: 3.0,
    });
    n.run(30.0, 0.0, |_, _| {});
    let want = (3.0 * 350.0 + 0.01 * 25.0) / 3.01;
    assert!(
        (n.t[tip] - want).abs() < 0.01,
        "{} against {want}",
        n.t[tip]
    );
}

#[test]
fn a_speck_is_in_balance_with_its_neighbors() {
    // a node far faster than a step: the weighted mean of its neighbors
    let mut n = net();
    let hot = n.node(10.0, 300.0, 0.0);
    let cold = n.node(10.0, 100.0, 0.0);
    let speck = n.node(1e-7, 25.0, 0.0);
    n.link(hot, speck, 0.3);
    n.link(cold, speck, 0.1);
    n.run(0.001, 0.0, |_, _| {});
    let want = (0.3 * n.t[hot] + 0.1 * n.t[cold]) / 0.4;
    assert!(
        (n.t[speck] - want).abs() < 0.01,
        "{} against {want}",
        n.t[speck]
    );
}
