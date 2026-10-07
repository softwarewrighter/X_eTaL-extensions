//! The scene model, headless: objects by id, the camera, rendering.

use xetal_ext_scene::model::{Kind, Object, Scene};

const BG: u32 = 0x000c_0e14;

fn line(points: Vec<[f64; 3]>) -> Object {
    Object {
        kind: Kind::Polyline,
        points,
        color: [1.0, 1.0, 1.0],
    }
}

#[test]
fn an_empty_scene_is_background() {
    assert!(Scene::default().render(8, 6).iter().all(|&p| p == BG));
}

#[test]
fn a_line_through_the_origin_crosses_the_centre() {
    let mut s = Scene::default();
    s.camera.yaw = 0.0;
    s.camera.pitch = 0.0;
    s.set(1, line(vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]));
    let (w, h) = (64, 48);
    let px = s.render(w, h);
    let lit = |x: usize, y: usize| px[y * w + x] != BG;
    // horizontal through the middle row, symmetric about the center
    assert!(lit(w / 2, h / 2) || lit(w / 2, h / 2 - 1));
    assert!(!lit(w / 2, 2) && !lit(w / 2, h - 3));
}

#[test]
fn ids_replace_and_remove() {
    let mut s = Scene::default();
    s.set(7, line(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]));
    s.set(7, line(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]));
    assert_eq!(s.objects.len(), 1);
    assert_eq!(s.objects[&7].points[1], [0.0, 1.0, 0.0]);
    assert!(s.remove(7));
    assert!(!s.remove(7));
}

#[test]
fn the_camera_turns_and_spins() {
    let mut s = Scene::default();
    let yaw = s.camera.yaw;
    s.drag(100.0, 1000.0);
    assert!((s.camera.yaw - (yaw + 1.0)).abs() < 1e-12);
    assert!((s.camera.pitch - 1.5).abs() < 1e-12, "pitch is clamped");
    s.camera.spin = 2.0;
    s.tick(0.5);
    assert!((s.camera.yaw - (yaw + 2.0)).abs() < 1e-12);
}

#[test]
fn points_behind_the_camera_are_skipped() {
    let mut s = Scene::default();
    s.camera.yaw = 0.0;
    s.camera.pitch = 0.0;
    s.camera.distance = 1.0;
    // z toward the viewer beyond the camera: nothing drawn, no panic
    s.set(1, line(vec![[0.0, 0.0, 5.0], [0.0, 0.1, 6.0]]));
    assert!(s.render(16, 16).iter().all(|&p| p == BG));
}

fn quad(points: Vec<[f64; 3]>, color: [f64; 3]) -> Object {
    Object {
        kind: Kind::Quads,
        points,
        color,
    }
}

/// A unit square facing the camera (yaw and pitch 0 look along z),
/// at depth z.
fn square(z: f64, size: f64) -> Vec<[f64; 3]> {
    vec![
        [-size, -size, z],
        [size, -size, z],
        [size, size, z],
        [-size, size, z],
    ]
}

fn straight_on() -> Scene {
    let mut s = Scene::default();
    s.camera.yaw = 0.0;
    s.camera.pitch = 0.0;
    s
}

#[test]
fn a_quad_fills_and_is_shaded() {
    let mut s = straight_on();
    s.set(1, quad(square(0.0, 0.5), [1.0, 0.0, 0.0]));
    let (w, h) = (40, 40);
    let px = s.render(w, h);
    let mid = px[(h / 2) * w + w / 2];
    assert_ne!(mid, BG, "the middle is covered");
    // facing along z, at an angle to the light: red, dimmed
    let red = (mid >> 16) & 0xff;
    assert!(red > 100 && red < 255, "{red}");
    assert_eq!(mid & 0xffff, 0, "only red");
    assert_eq!(px[0], BG, "the corner is not");
}

#[test]
fn the_nearer_quad_wins_whatever_the_order() {
    for order in [[1, 2], [2, 1]] {
        let mut s = straight_on();
        // the camera sits at +z (distance 4): larger z is nearer
        s.set(order[0], quad(square(0.5, 0.5), [0.0, 1.0, 0.0]));
        s.set(order[1], quad(square(-0.5, 0.8), [0.0, 0.0, 1.0]));
        let (w, h) = (40, 40);
        let px = s.render(w, h);
        let mid = px[(h / 2) * w + w / 2];
        assert!(
            (mid >> 8) & 0xff > 0 && mid & 0xff == 0,
            "green in front: {mid:06x}"
        );
        // beyond the green square's edge, the bigger blue one shows
        let side = px[(h / 2) * w + 4];
        assert!(side & 0xff > 0, "blue beside it: {side:06x}");
    }
}

#[test]
fn fog_fades_far_quads_and_lines_draw_over() {
    let mut s = straight_on();
    s.set(1, quad(square(0.0, 0.5), [1.0, 1.0, 1.0]));
    let clear = s.render(40, 40)[20 * 40 + 20];
    s.fog = Some((1.0, 5.0));
    let fogged = s.render(40, 40)[20 * 40 + 20];
    assert!(fogged & 0xff < clear & 0xff, "{fogged:06x} vs {clear:06x}");
    s.fog = Some((10.0, 20.0));
    assert_eq!(
        s.render(40, 40)[20 * 40 + 20],
        clear,
        "before the fog starts"
    );
    // a line across the quad is drawn on top
    s.set(2, line(vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]));
    let px = s.render(40, 40);
    assert!((19..=21).any(|y| px[y * 40 + 30] == 0x00ff_ffff || px[y * 40 + 30] != clear));
}

#[test]
fn a_quad_behind_the_camera_is_left_out() {
    let mut s = straight_on();
    s.set(1, quad(square(5.0, 0.5), [1.0, 1.0, 1.0]));
    assert!(s.render(20, 20).iter().all(|&p| p == BG));
}
