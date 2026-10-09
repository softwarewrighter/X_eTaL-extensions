//! The scene model, headless: objects by id, the camera, rendering.

use xetal_ext_scene::model::{Kind, Label, Object, Rect, Scene};

const BG: u32 = 0x000c_0e14;

fn line(points: Vec<[f64; 3]>) -> Object {
    Object {
        kind: Kind::Polyline,
        points,
        color: [1.0, 1.0, 1.0],
        alpha: 1.0,
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
        alpha: 1.0,
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

fn first_person(eye: [f64; 3], yaw: f64, pitch: f64) -> Scene {
    let mut s = Scene::default();
    s.camera.eye = Some(eye);
    s.camera.yaw = yaw;
    s.camera.pitch = pitch;
    s
}

#[test]
fn first_person_looks_along_yaw() {
    // a wall 5 ahead along -z: seen at yaw 0, not at yaw pi (behind)
    let wall = quad(square(-5.0, 1.0), [1.0, 1.0, 1.0]);
    let mut s = first_person([0.0, 0.0, 0.0], 0.0, 0.0);
    s.set(1, wall.clone());
    assert_ne!(s.render(40, 40)[20 * 40 + 20], BG);
    let mut s = first_person([0.0, 0.0, 0.0], std::f64::consts::PI, 0.0);
    s.set(1, wall);
    assert!(s.render(40, 40).iter().all(|&p| p == BG));
    // yaw pi/2 looks toward +x: a wall there is in the middle
    let east = quad(
        vec![
            [5.0, -1.0, -1.0],
            [5.0, -1.0, 1.0],
            [5.0, 1.0, 1.0],
            [5.0, 1.0, -1.0],
        ],
        [1.0, 1.0, 1.0],
    );
    let mut s = first_person([0.0, 0.0, 0.0], std::f64::consts::FRAC_PI_2, 0.0);
    s.set(1, east);
    assert_ne!(s.render(40, 40)[20 * 40 + 20], BG);
}

#[test]
fn a_floor_under_the_eye_is_clipped_not_dropped() {
    // a big floor 1.6 below the eye, reaching behind it: without
    // clipping at the near plane the whole quad would be left out
    let floor = quad(
        vec![
            [-20.0, 0.0, 20.0],
            [20.0, 0.0, 20.0],
            [20.0, 0.0, -20.0],
            [-20.0, 0.0, -20.0],
        ],
        [0.3, 0.8, 0.3],
    );
    let mut s = first_person([0.0, 1.6, 0.0], 0.0, -0.3);
    s.set(1, floor);
    let px = s.render(60, 40);
    assert_ne!(px[39 * 60 + 30], BG, "the bottom middle is floor");
    assert_eq!(px[0], BG, "the top corner is sky");
}

#[test]
fn looking_up_and_down() {
    let ceiling = quad(
        vec![
            [-5.0, 3.0, -5.0],
            [5.0, 3.0, -5.0],
            [5.0, 3.0, 5.0],
            [-5.0, 3.0, 5.0],
        ],
        [1.0, 1.0, 1.0],
    );
    let mut s = first_person([0.0, 0.0, 0.0], 0.0, 1.4);
    s.set(1, ceiling.clone());
    assert_ne!(s.render(40, 40)[20 * 40 + 20], BG, "up: the ceiling");
    let mut s = first_person([0.0, 0.0, 0.0], 0.0, -1.4);
    s.set(1, ceiling);
    assert!(s.render(40, 40).iter().all(|&p| p == BG), "down: nothing");
}

#[test]
fn objects_out_of_view_are_skipped_without_changing_the_picture() {
    let wall = quad(square(-5.0, 1.0), [1.0, 1.0, 1.0]);
    let behind = quad(square(5.0, 1.0), [1.0, 0.0, 0.0]);
    let mut s = first_person([0.0, 0.0, 0.0], 0.0, 0.0);
    s.set(1, wall);
    let alone = s.render(40, 40);
    s.set(2, behind);
    assert!(s.might_see(1, 40, 40));
    assert!(!s.might_see(2, 40, 40), "behind the eye");
    assert_eq!(s.render(40, 40), alone);
    // past the fog's end
    s.fog = Some((1.0, 3.0));
    assert!(
        !s.might_see(1, 40, 40),
        "the wall is 4 away, the fog ends at 3"
    );
    // orbiting sees everything
    s.camera.eye = None;
    assert!(s.might_see(2, 40, 40));
}

#[test]
fn a_curved_horizon_lowers_far_things_more() {
    // two walls of the same height, near and far: with the curve the far
    // one's top is lower on the screen than without it
    let top_row = |s: &Scene| {
        let px = s.render(60, 60);
        (0..60).find(|&y| px[y * 60 + 30] != BG).unwrap_or(60)
    };
    let far = quad(
        vec![
            [-20.0, -1.0, -40.0],
            [20.0, -1.0, -40.0],
            [20.0, 2.0, -40.0],
            [-20.0, 2.0, -40.0],
        ],
        [1.0, 1.0, 1.0],
    );
    let mut s = first_person([0.0, 0.0, 0.0], 0.0, 0.0);
    s.set(1, far);
    let flat = top_row(&s);
    s.camera.curve = 100.0;
    let curved = top_row(&s);
    assert!(curved > flat + 2, "flat {flat}, curved {curved}");
}

#[test]
fn the_overlay_is_drawn_over_everything_and_scaled_from_the_window() {
    // a white square at the center of a 40 by 30 window, over a red quad
    // that fills the view
    let mut s = first_person([0.0, 0.0, 0.0], 0.0, 0.0);
    s.set(
        1,
        quad(
            vec![
                [-50.0, -50.0, -2.0],
                [50.0, -50.0, -2.0],
                [50.0, 50.0, -2.0],
                [-50.0, 50.0, -2.0],
            ],
            [1.0, 0.0, 0.0],
        ),
    );
    s.size = (40.0, 30.0);
    s.overlay = vec![Rect {
        x: 18.0,
        y: 13.0,
        w: 4.0,
        h: 4.0,
        color: [1.0, 1.0, 1.0],
    }];
    let white = 0x00ff_ffff;
    // at the window's size: pixels 18 to 21 by 13 to 16
    let px = s.render(40, 30);
    assert_eq!(px[13 * 40 + 18], white);
    assert_eq!(px[16 * 40 + 21], white);
    assert_ne!(px[12 * 40 + 18], white);
    assert_ne!(px[13 * 40 + 22], white);
    // at twice the size (a dense screen): twice as far and as big
    let px = s.render(80, 60);
    assert_eq!(px[26 * 80 + 36], white);
    assert_eq!(px[33 * 80 + 43], white);
    assert_ne!(px[34 * 80 + 43], white);
    // cleared
    s.overlay.clear();
    assert!(s.render(40, 30).iter().all(|&p| p != white));
}

#[test]
fn a_translucent_quad_blends_over_what_is_behind_it_and_hides_nothing() {
    // a red wall behind a half-opaque blue pane: the middle is a mix;
    // a green wall in front of the pane still covers it
    let wall = |z: f64, color: [f64; 3]| {
        quad(
            vec![
                [-50.0, -50.0, z],
                [50.0, -50.0, z],
                [50.0, 50.0, z],
                [-50.0, 50.0, z],
            ],
            color,
        )
    };
    let mut s = first_person([0.0, 0.0, 0.0], 0.0, 0.0);
    s.set(1, wall(-4.0, [1.0, 0.0, 0.0]));
    let red = s.render(20, 20)[10 * 20 + 10];
    let mut pane = wall(-2.0, [0.0, 0.0, 1.0]);
    pane.alpha = 0.5;
    s.set(2, pane);
    let mixed = s.render(20, 20)[10 * 20 + 10];
    let (r, b) = ((mixed >> 16) & 0xff, mixed & 0xff);
    assert!(r > 0 && b > 0, "both show through: {mixed:06x}");
    assert!(r < (red >> 16) & 0xff, "the red is dimmed");
    // drawn after the opaque quads whatever the ids, and never hiding them
    s.set(0, wall(-1.0, [0.0, 1.0, 0.0]));
    let front = s.render(20, 20)[10 * 20 + 10];
    assert_eq!(front & 0xff, 0, "no blue over the nearer green wall");
    assert!((front >> 8) & 0xff > 0);
}

#[test]
fn a_label_draws_its_glyphs_over_the_scene() {
    // "U'" at the top left, 7 logical pixels tall: one picture pixel a
    // font pixel in a 20 by 10 window drawn at 20 by 10
    let mut s = Scene {
        size: (20.0, 10.0),
        ..Scene::default()
    };
    s.labels.insert(
        1,
        Label {
            x: 0.0,
            y: 0.0,
            size: 7.0,
            color: [1.0, 1.0, 1.0],
            text: "U'".into(),
        },
    );
    let px = s.render(20, 10);
    let lit = |x: usize, y: usize| px[y * 20 + x] == 0x00ff_ffff;
    // U: the sides down, the bottom curved in; nothing in the middle
    assert!(lit(0, 0) && lit(4, 0) && !lit(2, 0));
    assert!(lit(1, 6) && lit(3, 6) && !lit(0, 6));
    // the prime after one blank column: 6 + 2 and 6 + 3 on the top row
    assert!(lit(8, 0) && lit(9, 0) && !lit(5, 0));
    // removed, nothing
    s.labels.clear();
    assert!(s.render(20, 10).iter().all(|&p| p == BG));
}
