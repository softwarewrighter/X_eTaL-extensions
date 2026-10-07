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

fn locked(s: &Shared) -> std::sync::MutexGuard<'_, Scene> {
    s.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct Pane {
    window: Arc<Window>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    scene: Shared,
    events: Events,
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
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                if let Some(k) = key_name(&event.logical_key) {
                    self.events.push(format!("key {k}"));
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => self.dragging = *state == ElementState::Pressed,
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                if self.dragging {
                    locked(&self.scene).drag(x - self.cursor.0, y - self.cursor.1);
                    self.window.request_redraw();
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
    static NEXT: AtomicI64 = AtomicI64::new(1);
    let (window, events) = if headless() {
        (None, scripted())
    } else {
        let events = Events::new();
        let (pane_events, pane_scene) = (events.clone(), scene.clone());
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
        next: 1, "Num a => a -> Char", "The next event (frame, key NAME, close), waiting up to a frame.";
        close: 1, "Num a => a -> Int", "Close the window.";
    }
}
