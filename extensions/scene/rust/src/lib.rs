//! scene: retained 3D line and point scenes in a native window (plan
//! M2), patched by id from X_eTaL, drawn on the CPU (`model`), served on
//! the host's main thread (`xetal-ext-ui`, plan M1). The mouse orbits
//! the camera; the program pulls events. Without a screen:
//! `XETAL_HEADLESS=1` (events from `XETAL_EVENTS`) and
//! `XETAL_FRAMES=DIR` (each frame saved as `DIR/scene-ID-N.png`).

pub mod model;

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use model::{Kind, Object, Scene};
use xetal_ext_sdk::{OwnedError, Value, float_vector, text};
use xetal_ext_ui::winit::dpi::LogicalSize;
use xetal_ext_ui::winit::event::{ElementState, MouseButton, WindowEvent};
use xetal_ext_ui::winit::keyboard::{Key, NamedKey};
use xetal_ext_ui::winit::window::{Window, WindowId};
use xetal_ext_ui::{Events, Surface, frames_dir, headless, on_main, save_png, scripted};

type Shared = Arc<Mutex<Scene>>;

/// What the player is doing with the keyboard and mouse: the keys held
/// down (by name, lower case) and how far the mouse was dragged since
/// the program last asked -- for a first-person camera, which looks
/// where it is dragged (the pointer is never grabbed).
#[derive(Default)]
pub struct Input {
    held: std::collections::BTreeSet<String>,
    look: (f64, f64),
}

/// The keys `sc:c_ontrols` reports, in order.
pub const CONTROL_KEYS: [&str; 10] = [
    "w",
    "a",
    "s",
    "d",
    "space",
    "shift",
    "arrowleft",
    "arrowright",
    "arrowup",
    "arrowdown",
];

impl Input {
    /// Applies a scripted event (`keydown w`, `keyup w`, `drag 10 -4`);
    /// whether it was one.
    fn script(&mut self, event: &str) -> bool {
        let mut words = event.split_whitespace();
        match (words.next(), words.next(), words.next()) {
            (Some("keydown"), Some(k), None) => {
                self.held.insert(k.to_lowercase());
                true
            }
            (Some("keyup"), Some(k), None) => {
                self.held.remove(&k.to_lowercase());
                true
            }
            (Some("drag"), Some(x), Some(y)) => {
                if let (Ok(x), Ok(y)) = (x.parse::<f64>(), y.parse::<f64>()) {
                    self.look.0 += x;
                    self.look.1 += y;
                }
                true
            }
            _ => false,
        }
    }

    /// The mouse's drag since last asked (then cleared) and 1 or 0 for
    /// each of [`CONTROL_KEYS`].
    fn take(&mut self) -> Vec<f64> {
        let mut v = vec![self.look.0, self.look.1];
        self.look = (0.0, 0.0);
        v.extend(
            CONTROL_KEYS
                .iter()
                .map(|k| if self.held.contains(*k) { 1.0 } else { 0.0 }),
        );
        v
    }
}

type SharedInput = Arc<Mutex<Input>>;

fn input_of(i: &SharedInput) -> std::sync::MutexGuard<'_, Input> {
    i.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn locked(s: &Shared) -> std::sync::MutexGuard<'_, Scene> {
    s.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct Pane {
    window: Arc<Window>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    scene: Shared,
    events: Events,
    input: SharedInput,
    cursor: (f64, f64),
    dragging: bool,
}

impl Pane {
    fn draw(&mut self) {
        let size = self.window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return;
        };
        if self.surface.resize(w, h).is_err() {
            return;
        }
        let pixels = locked(&self.scene).render(w.get() as usize, h.get() as usize);
        let Ok(mut buffer) = self.surface.buffer_mut() else {
            return;
        };
        buffer.copy_from_slice(&pixels);
        let _ = buffer.present();
    }
}

fn key_name(k: &Key) -> Option<String> {
    match k {
        Key::Named(NamedKey::Space) => Some("Space".into()),
        Key::Named(n) => Some(format!("{n:?}")),
        Key::Character(c) => Some(c.to_string()),
        _ => None,
    }
}

impl Surface for Pane {
    fn event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::CloseRequested => self.events.push("close"),
            WindowEvent::KeyboardInput { event, .. } => {
                let Some(k) = key_name(&event.logical_key) else {
                    return;
                };
                if event.state == ElementState::Pressed {
                    if !event.repeat {
                        self.events.push(format!("key {k}"));
                    }
                    input_of(&self.input).held.insert(k.to_lowercase());
                } else {
                    input_of(&self.input).held.remove(&k.to_lowercase());
                }
            }
            // keys let go while the window was in the background
            WindowEvent::Focused(false) => input_of(&self.input).held.clear(),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => self.dragging = *state == ElementState::Pressed,
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                if self.dragging {
                    let (dx, dy) = (x - self.cursor.0, y - self.cursor.1);
                    let mut scene = locked(&self.scene);
                    if scene.camera.eye.is_some() {
                        // first person: the program turns the camera
                        let mut i = input_of(&self.input);
                        i.look.0 += dx;
                        i.look.1 += dy;
                    } else {
                        scene.drag(dx, dy);
                        self.window.request_redraw();
                    }
                }
                self.cursor = (x, y);
            }
            _ => {}
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// The program's side of a scene window.
struct Handle {
    window: Option<WindowId>,
    scene: Shared,
    events: Events,
    input: SharedInput,
    size: (usize, usize),
    shown: usize,
    last: Instant,
}

fn handles() -> std::sync::MutexGuard<'static, HashMap<i64, Handle>> {
    static H: OnceLock<Mutex<HashMap<i64, Handle>>> = OnceLock::new();
    H.get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[allow(clippy::cast_possible_truncation)]
fn int(v: f64) -> i64 {
    v.round() as i64
}

fn id_of(v: &Value) -> Result<i64, OwnedError> {
    let (_, x) = float_vector(v)?;
    x.first()
        .copied()
        .map(int)
        .ok_or_else(|| OwnedError::invalid_argument("a scene id (an Int from open)"))
}

fn with<R>(id: i64, f: impl FnOnce(&mut Handle) -> R) -> Result<R, OwnedError> {
    handles()
        .get_mut(&id)
        .map(f)
        .ok_or_else(|| OwnedError::invalid_argument(format!("no scene {id} is open")))
}

fn redraw(window: Option<WindowId>) -> Result<(), OwnedError> {
    if let Some(window) = window {
        on_main(move |ui| {
            if let Some(p) = ui
                .surface(window)
                .and_then(|s| s.as_any().downcast_mut::<Pane>())
            {
                p.window.request_redraw();
            }
        })
        .map_err(OwnedError::failure)?;
    }
    Ok(())
}

/// title open size: a scene window `width height` points; its id.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn open(args: &[Value]) -> Result<Value, OwnedError> {
    let title = text(&args[0])?;
    let (_, size) = float_vector(&args[1])?;
    let [w, h] = size[..] else {
        return Err(OwnedError::invalid_argument("size is width height"));
    };
    let scene: Shared = Arc::new(Mutex::new(Scene::default()));
    let input: SharedInput = Arc::new(Mutex::new(Input::default()));
    static NEXT: AtomicI64 = AtomicI64::new(1);
    let (window, events) = if headless() {
        (None, scripted())
    } else {
        let events = Events::new();
        let (pane_events, pane_scene, pane_input) = (events.clone(), scene.clone(), input.clone());
        let window = on_main(move |ui| {
            let attrs = Window::default_attributes()
                .with_title(title)
                .with_inner_size(LogicalSize::new(w, h));
            ui.open(attrs, move |window| {
                let context =
                    softbuffer::Context::new(window.clone()).map_err(|e| e.to_string())?;
                let surface = softbuffer::Surface::new(&context, window.clone())
                    .map_err(|e| e.to_string())?;
                window.request_redraw();
                Ok(Box::new(Pane {
                    window,
                    surface,
                    scene: pane_scene,
                    events: pane_events,
                    input: pane_input,
                    cursor: (0.0, 0.0),
                    dragging: false,
                }))
            })
        })
        .map_err(OwnedError::failure)?
        .map_err(OwnedError::failure)?;
        (Some(window), events)
    };
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    handles().insert(
        id,
        Handle {
            window,
            scene,
            events,
            input,
            size: (w.max(1.0) as usize, h.max(1.0) as usize),
            shown: 0,
            last: Instant::now(),
        },
    );
    Ok(Value::Int(id))
}

/// The header of an object: scene id, object id, red, green, blue
/// (0 to 1).
fn header(v: &Value) -> Result<(i64, i64, [f64; 3]), OwnedError> {
    let (_, h) = float_vector(v)?;
    match h[..] {
        [s, o, r, g, b] => Ok((int(s), int(o), [r, g, b])),
        [s, o] => Ok((int(s), int(o), [0.9, 0.9, 0.9])),
        _ => Err(OwnedError::invalid_argument(
            "header: scene object, or scene object red green blue",
        )),
    }
}

/// Points: an n by 3 matrix (or one point, 3 numbers).
fn points(v: &Value) -> Result<Vec<[f64; 3]>, OwnedError> {
    let (shape, x) = float_vector(v)?;
    match shape[..] {
        [_, 3] | [3] => Ok(x.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()),
        _ => Err(OwnedError::invalid_argument(format!(
            "points are n by 3, not {shape:?}"
        ))),
    }
}

fn put(kind: Kind, args: &[Value]) -> Result<Value, OwnedError> {
    let (s, o, color) = header(&args[0])?;
    let pts = points(&args[1])?;
    let window = with(s, |h| {
        locked(&h.scene).set(
            o,
            Object {
                kind,
                points: pts,
                color,
            },
        );
        h.window
    })?;
    redraw(window)?;
    Ok(Value::Int(o))
}

fn polyline(args: &[Value]) -> Result<Value, OwnedError> {
    put(Kind::Polyline, args)
}

fn segments(args: &[Value]) -> Result<Value, OwnedError> {
    put(Kind::Segments, args)
}

fn dots(args: &[Value]) -> Result<Value, OwnedError> {
    put(Kind::Points, args)
}

/// header quads points: taken in fours, each a filled quad.
fn quads(args: &[Value]) -> Result<Value, OwnedError> {
    let (shape, _) = float_vector(&args[1])?;
    if shape.first().is_some_and(|n| n % 4 != 0) || shape.len() != 2 {
        return Err(OwnedError::invalid_argument(format!(
            "quads are 4n by 3 points (four corners each), not {shape:?}"
        )));
    }
    put(Kind::Quads, args)
}

/// scene fog near far: quads fade into the background from depth near
/// to far (scene units); far <= near turns fog off. The scene id.
fn fog(args: &[Value]) -> Result<Value, OwnedError> {
    let s = id_of(&args[0])?;
    let (_, f) = float_vector(&args[1])?;
    let [near, far] = f[..] else {
        return Err(OwnedError::invalid_argument("fog is near far"));
    };
    let window = with(s, |h| {
        locked(&h.scene).fog = (far > near).then_some((near, far));
        h.window
    })?;
    redraw(window)?;
    Ok(Value::Int(s))
}

/// scene remove object: 1 if it was there, else 0.
fn remove(args: &[Value]) -> Result<Value, OwnedError> {
    let s = id_of(&args[0])?;
    let o = id_of(&args[1])?;
    let (gone, window) = with(s, |h| (locked(&h.scene).remove(o), h.window))?;
    redraw(window)?;
    Ok(Value::Int(i64::from(gone)))
}

/// scene camera yaw pitch distance spin (radians, units, radians per
/// second); the scene id.
fn camera(args: &[Value]) -> Result<Value, OwnedError> {
    let s = id_of(&args[0])?;
    let (_, c) = float_vector(&args[1])?;
    let [yaw, pitch, distance, spin] = c[..] else {
        return Err(OwnedError::invalid_argument(
            "camera is yaw pitch distance spin",
        ));
    };
    let window = with(s, |h| {
        let mut sc = locked(&h.scene);
        sc.camera.yaw = yaw;
        sc.camera.pitch = pitch.clamp(-1.5, 1.5);
        sc.camera.distance = distance.max(0.1);
        sc.camera.spin = spin;
        sc.camera.eye = None;
        h.window
    })?;
    redraw(window)?;
    Ok(Value::Int(s))
}

/// next scene: the next event, waiting up to a frame (`frame`,
/// `key NAME`, `close`); auto-rotation advances; with `XETAL_FRAMES`
/// the frame is saved.
fn next(args: &[Value]) -> Result<Value, OwnedError> {
    let s = id_of(&args[0])?;
    let events = with(s, |h| h.events.clone())?;
    let event = events.next(Duration::from_micros(16_667));
    with(s, |h| input_of(&h.input).script(&event))?;
    let (window, save) = with(s, |h| {
        let dt = if h.window.is_none() {
            1.0 / 60.0
        } else {
            h.last.elapsed().as_secs_f64().min(0.25)
        };
        h.last = Instant::now();
        locked(&h.scene).tick(dt);
        h.shown += 1;
        let save = frames_dir().map(|d| {
            let (w, ht) = h.size;
            (
                d.join(format!("scene-{s}-{}.png", h.shown)),
                w,
                ht,
                locked(&h.scene).render(w, ht),
            )
        });
        (h.window, save)
    })?;
    if let Some((path, w, h, pixels)) = save {
        save_png(w, h, &pixels, &path).map_err(OwnedError::failure)?;
    }
    redraw(window)?;
    Ok(Value::Text(event))
}

/// scene eye x y z yaw pitch: a first-person camera at the eye,
/// looking along yaw (0 toward -z, right toward +x) and pitch (up
/// positive), radians; `sc:c_amera!` goes back to orbiting. The scene id.
fn eye(args: &[Value]) -> Result<Value, OwnedError> {
    let s = id_of(&args[0])?;
    let (_, e) = float_vector(&args[1])?;
    let [x, y, z, yaw, pitch] = e[..] else {
        return Err(OwnedError::invalid_argument("eye is x y z yaw pitch"));
    };
    if e.iter().any(|v| !v.is_finite()) {
        return Err(OwnedError::invalid_argument("eye: a NaN or an infinity"));
    }
    let window = with(s, |h| {
        let mut sc = locked(&h.scene);
        sc.camera.eye = Some([x, y, z]);
        sc.camera.yaw = yaw;
        sc.camera.pitch = pitch.clamp(-1.55, 1.55);
        sc.camera.spin = 0.0;
        h.window
    })?;
    redraw(window)?;
    Ok(Value::Int(s))
}

/// scene curve radius: a curved horizon for the first-person camera,
/// the world lowered by d^2 / 2R at distance d along the ground; 0 turns
/// it off. The scene id.
fn curve(args: &[Value]) -> Result<Value, OwnedError> {
    let s = id_of(&args[0])?;
    let (_, r) = float_vector(&args[1])?;
    let [r] = r[..] else {
        return Err(OwnedError::invalid_argument("curve is one radius"));
    };
    if !r.is_finite() || r < 0.0 {
        return Err(OwnedError::invalid_argument(format!("{r} is not a radius")));
    }
    let window = with(s, |h| {
        locked(&h.scene).camera.curve = r;
        h.window
    })?;
    redraw(window)?;
    Ok(Value::Int(s))
}

/// scene sky red green blue: the background, which fog fades to (0 to
/// 1); -1 -1 -1, the dark default. The scene id.
fn sky(args: &[Value]) -> Result<Value, OwnedError> {
    let s = id_of(&args[0])?;
    let (_, c) = float_vector(&args[1])?;
    let [r, g, b] = c[..] else {
        return Err(OwnedError::invalid_argument("sky is red green blue"));
    };
    let window = with(s, |h| {
        locked(&h.scene).sky = (r >= 0.0).then_some([r, g, b]);
        h.window
    })?;
    redraw(window)?;
    Ok(Value::Int(s))
}

/// controls scene: how far the mouse was dragged since last asked (x,
/// y pixels), then 1 or 0 for each key held: w a s d space shift and the
/// four arrows.
fn controls(args: &[Value]) -> Result<Value, OwnedError> {
    let s = id_of(&args[0])?;
    let v = with(s, |h| input_of(&h.input).take())?;
    xetal_ext_sdk::Array::new(vec![v.len()], xetal_ext_sdk::ArrayData::Float(v))
        .map(Value::Array)
        .map_err(|e| OwnedError::failure(e.to_string()))
}

/// close scene: close the window; 1.
fn close(args: &[Value]) -> Result<Value, OwnedError> {
    let s = id_of(&args[0])?;
    let window = with(s, |h| h.window)?;
    handles().remove(&s);
    if let Some(window) = window {
        on_main(move |ui| ui.close(window)).map_err(OwnedError::failure)?;
    }
    Ok(Value::Int(1))
}

xetal_ext_sdk::xetal_extension! {
    name: "scene",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        open: 2, "Num a => Char -> a -> Int", "title open width height: a 3D scene window; its id.";
        polyline: 2, "(Num a, Num b) => a -> b -> Int", "scene object red green blue polyline points (n by 3): joined in order.";
        segments: 2, "(Num a, Num b) => a -> b -> Int", "header segments points: taken in pairs.";
        dots: 2, "(Num a, Num b) => a -> b -> Int", "header dots points: each a dot.";
        quads: 2, "(Num a, Num b) => a -> b -> Int", "header quads points (4n by 3): filled, shaded, depth-tested.";
        fog: 2, "(Num a, Num b) => a -> b -> Int", "scene fog near far: quads fade into the background.";
        remove: 2, "(Num a, Num b) => a -> b -> Int", "scene remove object: 1 if it was there.";
        camera: 2, "(Num a, Num b) => a -> b -> Int", "scene camera yaw pitch distance spin.";
        eye: 2, "(Num a, Num b) => a -> b -> Int", "scene eye x y z yaw pitch: a first-person camera.";
        sky: 2, "(Num a, Num b) => a -> b -> Int", "scene sky red green blue: the background and fog color.";
        curve: 2, "(Num a, Num b) => a -> b -> Int", "scene curve radius: a curved horizon (0: flat).";
        controls: 1, "Num a => a -> Float", "Mouse drag dx dy since last asked, then held: w a s d space shift left right up down.";
        next: 1, "Num a => a -> Char", "The next event (frame, key NAME, close), waiting up to a frame.";
        close: 1, "Num a => a -> Int", "Close the window.";
    }
}
