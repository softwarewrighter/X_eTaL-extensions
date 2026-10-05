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
