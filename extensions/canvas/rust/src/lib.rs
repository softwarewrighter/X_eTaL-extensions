//! canvas: a native window showing X_eTaL arrays as pixels, with keys,
//! clicks and frame ticks back. Linked into the host, which serves its
//! windows on the main thread (`xetal-ext-ui`, plan M1); drawn with
//! softbuffer (a CPU pixel buffer in a winit window, plan M7).
//!
//! Without a screen: `XETAL_HEADLESS=1` opens no window and plays the
//! events in `XETAL_EVENTS` (separated by commas: `frame,key q`), then
//! `close`; `XETAL_FRAMES=DIR` saves every frame shown as
//! `DIR/canvas-ID-N.png`, with or without a window (helpers in
//! `xetal-ext-ui`).

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use xetal_ext_sdk::{ArrayData, OwnedError, Value, text};
use xetal_ext_ui::winit::dpi::LogicalSize;
use xetal_ext_ui::winit::event::{ElementState, MouseButton, WindowEvent};
use xetal_ext_ui::winit::keyboard::{Key, NamedKey};
use xetal_ext_ui::winit::window::{Window, WindowId};
use xetal_ext_ui::{Events, Surface, headless, on_main, scripted};

/// A picture: width, height and 0RGB pixels, row by row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

fn rgb(r: u32, g: u32, b: u32) -> u32 {
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 0..=255
fn byte_f(x: f64) -> u32 {
    (x.clamp(0.0, 1.0) * 255.0).round() as u32
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn byte_i(x: i64) -> u32 {
    x.clamp(0, 255) as u32
}

/// An array as a frame: an n by m matrix is gray (Bool: 1 white;
/// Int 0..255; Float 0..1), an n by m by 3 array is color (red, green,
/// blue in the last axis, Int 0..255 or Float 0..1).
///
/// # Errors
///
/// Any other shape or element type.
pub fn frame(v: &Value) -> Result<Frame, OwnedError> {
    let Value::Array(a) = v else {
        return Err(OwnedError::invalid_argument(
            "show a matrix (gray) or an n by m by 3 array (color)",
        ));
    };
    let s = a.shape();
    let (h, w, color) = match s {
        [h, w] => (*h, *w, false),
        [h, w, 3] => (*h, *w, true),
        _ => {
            return Err(OwnedError::invalid_argument(format!(
                "shape {s:?}: show n by m, or n by m by 3"
            )));
        }
    };
    let gray: Vec<u32> = match a.data() {
        ArrayData::Bool(b) => b.iter().map(|&b| if b { 255 } else { 0 }).collect(),
        ArrayData::Int(i) => i.iter().map(|&x| byte_i(x)).collect(),
        ArrayData::Float(f) => f.iter().map(|&x| byte_f(x)).collect(),
        ArrayData::Char(_) => return Err(OwnedError::invalid_argument("show numbers, not text")),
    };
    let pixels = if color {
        gray.chunks_exact(3)
            .map(|c| rgb(c[0], c[1], c[2]))
            .collect()
    } else {
        gray.iter().map(|&g| rgb(g, g, g)).collect()
    };
    Ok(Frame {
        width: w,
        height: h,
        pixels,
    })
}

/// `frame` scaled to `w` by `h` (nearest pixel; letterboxed to keep its
/// shape), and where a window pixel falls in the frame.
pub fn scaled(f: &Frame, w: usize, h: usize) -> Vec<u32> {
    let mut out = vec![0x0010_1214; w * h];
    if f.width == 0 || f.height == 0 {
        return out;
    }
    let (s, ox, oy) = fit(f, w, h);
    for y in 0..h {
        for x in 0..w {
            if let Some((r, c)) = at(f, s, ox, oy, x, y) {
                out[y * w + x] = f.pixels[r * f.width + c];
            }
        }
    }
    out
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn fit(f: &Frame, w: usize, h: usize) -> (f64, f64, f64) {
    let s = (w as f64 / f.width as f64).min(h as f64 / f.height as f64);
    let ox = (w as f64 - s * f.width as f64) / 2.0;
    let oy = (h as f64 - s * f.height as f64) / 2.0;
    (s, ox, oy)
}

/// The frame cell (row, column) under window pixel (x, y), if any.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn at(f: &Frame, s: f64, ox: f64, oy: f64, x: usize, y: usize) -> Option<(usize, usize)> {
    let c = ((x as f64 + 0.5 - ox) / s).floor();
    let r = ((y as f64 + 0.5 - oy) / s).floor();
    (c >= 0.0 && r >= 0.0 && (c as usize) < f.width && (r as usize) < f.height)
        .then_some((r as usize, c as usize))
}

/// A key as the program sees it: `Space`, `Escape`, `ArrowLeft`, `q`.
fn key_name(k: &Key) -> Option<String> {
    match k {
        Key::Named(NamedKey::Space) => Some("Space".into()),
        Key::Named(n) => Some(format!("{n:?}")),
        Key::Character(c) => Some(c.to_string()),
        _ => None,
    }
}

struct Canvas {
    window: Arc<Window>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    frame: Frame,
    events: Events,
    cursor: (f64, f64),
}

impl Canvas {
    fn draw(&mut self) {
        let size = self.window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return;
        };
        if self.surface.resize(w, h).is_err() {
            return;
        }
        let Ok(mut buffer) = self.surface.buffer_mut() else {
            return;
        };
        let pixels = scaled(&self.frame, w.get() as usize, h.get() as usize);
        buffer.copy_from_slice(&pixels);
        let _ = buffer.present();
    }
}

impl Surface for Canvas {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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
            WindowEvent::CursorMoved { position, .. } => self.cursor = (position.x, position.y),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let size = self.window.inner_size();
                let (s, ox, oy) = fit(&self.frame, size.width as usize, size.height as usize);
                let (x, y) = (
                    self.cursor.0.max(0.0) as usize,
                    self.cursor.1.max(0.0) as usize,
                );
                if let Some((r, c)) = at(&self.frame, s, ox, oy, x, y) {
                    self.events.push(format!("click {} {}", r + 1, c + 1));
                }
            }
            _ => {}
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// The program's side of a canvas: its window (none when headless),
/// its events, and how many frames it has shown.
struct Handle {
    window: Option<WindowId>,
    events: Events,
    shown: usize,
}

/// Writes a frame as an RGB PNG.
///
/// # Errors
///
/// When the file cannot be written.
pub fn save_png(f: &Frame, path: &std::path::Path) -> Result<(), String> {
    xetal_ext_ui::save_png(f.width, f.height, &f.pixels, path)
}

fn canvases() -> &'static Mutex<HashMap<i64, Handle>> {
    static C: OnceLock<Mutex<HashMap<i64, Handle>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

fn lock() -> std::sync::MutexGuard<'static, HashMap<i64, Handle>> {
    canvases()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn id_of(v: &Value) -> Result<i64, OwnedError> {
    match v {
        Value::Int(i) => Ok(*i),
        #[allow(clippy::cast_possible_truncation)]
        Value::Float(x) if x.fract() == 0.0 => Ok(*x as i64),
        _ => Err(OwnedError::invalid_argument(
            "a canvas id (an Int from open)",
        )),
    }
}

fn handle(id: i64) -> Result<(Option<WindowId>, Events), OwnedError> {
    lock()
        .get(&id)
        .map(|h| (h.window, h.events.clone()))
        .ok_or_else(|| OwnedError::invalid_argument(format!("no canvas {id} is open")))
}

fn host_error(e: String) -> OwnedError {
    OwnedError::failure(e)
}

/// title open size: a window of `width height` (logical pixels); its id.
fn open(args: &[Value]) -> Result<Value, OwnedError> {
    let title = text(&args[0])?;
    let (_, size) = xetal_ext_sdk::float_vector(&args[1])?;
    let [w, h] = size[..] else {
        return Err(OwnedError::invalid_argument("size is width height"));
    };
    static NEXT: AtomicI64 = AtomicI64::new(1);
    if headless() {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        lock().insert(
            id,
            Handle {
                window: None,
                events: scripted(),
                shown: 0,
            },
        );
        return Ok(Value::Int(id));
    }
    let events = Events::new();
    let main_events = events.clone();
    let window = on_main(move |ui| {
        let attrs = Window::default_attributes()
            .with_title(title)
            .with_inner_size(LogicalSize::new(w, h));
        ui.open(attrs, move |window| {
            let context = softbuffer::Context::new(window.clone()).map_err(|e| e.to_string())?;
            let surface =
                softbuffer::Surface::new(&context, window.clone()).map_err(|e| e.to_string())?;
            window.request_redraw();
            Ok(Box::new(Canvas {
                window,
                surface,
                frame: Frame {
                    width: 0,
                    height: 0,
                    pixels: Vec::new(),
                },
                events: main_events,
                cursor: (0.0, 0.0),
            }))
        })
    })
    .map_err(host_error)?
    .map_err(host_error)?;
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    lock().insert(
        id,
        Handle {
            window: Some(window),
            events,
            shown: 0,
        },
    );
    Ok(Value::Int(id))
}

/// id show array: draw the array in the canvas (see `frame`); the id.
fn show(args: &[Value]) -> Result<Value, OwnedError> {
    let id = id_of(&args[0])?;
    let (window, _) = handle(id)?;
    let f = frame(&args[1])?;
    if let Some(dir) = xetal_ext_ui::frames_dir() {
        let n = {
            let mut c = lock();
            let h = c
                .get_mut(&id)
                .ok_or_else(|| OwnedError::invalid_argument(format!("no canvas {id} is open")))?;
            h.shown += 1;
            h.shown
        };
        if xetal_ext_ui::frame_wanted(u64::try_from(n).unwrap_or(u64::MAX)) {
            save_png(&f, &dir.join(format!("canvas-{id}-{n}.png"))).map_err(OwnedError::failure)?;
        }
    }
    let Some(window) = window else {
        return Ok(Value::Int(id));
    };
    on_main(move |ui| {
        if let Some(canvas) = ui
            .surface(window)
            .and_then(|s| s.as_any().downcast_mut::<Canvas>())
        {
            canvas.frame = f;
            canvas.window.request_redraw();
        }
    })
    .map_err(host_error)?;
    Ok(Value::Int(id))
}

/// next id: the next event, waiting up to a frame (1/60 s): `frame`,
/// `key NAME`, `click ROW COL` (1-based, in the array shown), `close`.
fn next(args: &[Value]) -> Result<Value, OwnedError> {
    let (_, events) = handle(id_of(&args[0])?)?;
    Ok(Value::Text(events.next(Duration::from_micros(16_667))))
}

/// close id: close the window; 1.
fn close(args: &[Value]) -> Result<Value, OwnedError> {
    let id = id_of(&args[0])?;
    let (window, _) = handle(id)?;
    lock().remove(&id);
    if let Some(window) = window {
        on_main(move |ui| ui.close(window)).map_err(host_error)?;
    }
    Ok(Value::Int(1))
}

xetal_ext_sdk::xetal_extension! {
    name: "canvas",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        open: 2, "Char -> Int -> Int", "title open width height: a window; its id.";
        show: 2, "Num a => Int -> a -> Int", "id show array: draw a gray matrix or an n by m by 3 color array.";
        next: 1, "Int -> Char", "The next event (frame, key NAME, click ROW COL, close), waiting up to a frame.";
        close: 1, "Int -> Int", "Close the window.";
    }
}
