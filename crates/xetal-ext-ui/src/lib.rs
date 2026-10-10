//! The UI host (plan M1). On macOS a window must live on the main
//! thread, but X_eTaL runs a program on a worker thread. So the host
//! (`xetal-x`) runs the program on another thread and calls [`serve`]
//! on the main thread; a UI extension, linked into the host, asks the
//! main thread for work with [`on_main`] and reads the events its
//! [`Surface`] records from the program's thread (see [`Events`]).
//!
//! The winit event loop is created only when the first job arrives, so
//! a program that opens no window runs exactly as without a UI host.

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{Window, WindowAttributes, WindowId};

pub use winit;

/// What a window does on the main thread: draw, and turn window events
/// into the events its program reads.
pub trait Surface {
    /// A window event for this surface's window.
    fn event(&mut self, event: &WindowEvent);

    /// The surface itself, so its extension can reach its own type
    /// (`ui.surface(id)?.as_any().downcast_mut::<Mine>()`).
    fn as_any(&mut self) -> &mut dyn std::any::Any;
}

/// The main thread's side, handed to a job: open windows, reach
/// surfaces.
pub struct Ui<'a> {
    event_loop: &'a ActiveEventLoop,
    surfaces: &'a mut HashMap<WindowId, Box<dyn Surface>>,
}

impl Ui<'_> {
    /// Opens a window and registers the surface `make` builds for it.
    ///
    /// # Errors
    ///
    /// When the platform refuses the window, or `make` fails.
    pub fn open(
        &mut self,
        attributes: WindowAttributes,
        make: impl FnOnce(Arc<Window>) -> Result<Box<dyn Surface>, String>,
    ) -> Result<WindowId, String> {
        let window = Arc::new(
            self.event_loop
                .create_window(attributes)
                .map_err(|e| e.to_string())?,
        );
        let id = window.id();
        let surface = make(window)?;
        self.surfaces.insert(id, surface);
        Ok(id)
    }

    /// The surface of a window, to change it.
    pub fn surface(&mut self, id: WindowId) -> Option<&mut Box<dyn Surface>> {
        self.surfaces.get_mut(&id)
    }

    /// Closes a window (dropping its surface closes it).
    pub fn close(&mut self, id: WindowId) {
        self.surfaces.remove(&id);
    }
}

type Job = Box<dyn FnOnce(&mut Ui<'_>) + Send>;

struct Host {
    jobs: Sender<Job>,
    wake: Mutex<Option<EventLoopProxy<()>>>,
}

static HOST: OnceLock<Host> = OnceLock::new();

/// Runs `job` on the main thread and returns its result, waiting for
/// it.
///
/// # Errors
///
/// When no UI host is serving (the program is not run by a host that
/// calls [`serve`]), or the host stopped.
pub fn on_main<R: Send + 'static>(
    job: impl FnOnce(&mut Ui<'_>) -> R + Send + 'static,
) -> Result<R, String> {
    let host = HOST
        .get()
        .ok_or("no UI host: windows need xetal-x (or another host serving the main thread)")?;
    let (reply, answer) = mpsc::channel();
    host.jobs
        .send(Box::new(move |ui| {
            let _ = reply.send(job(ui));
        }))
        .map_err(|_| "the UI host has stopped".to_string())?;
    if let Some(proxy) = host
        .wake
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
    {
        let _ = proxy.send_event(());
    }
    answer
        .recv()
        .map_err(|_| "the UI host has stopped".to_string())
}

/// Whether a UI host is serving in this process.
pub fn hosted() -> bool {
    HOST.get().is_some()
}

struct App {
    jobs: Receiver<Job>,
    surfaces: HashMap<WindowId, Box<dyn Surface>>,
}

impl App {
    fn drain(&mut self, event_loop: &ActiveEventLoop) {
        while let Ok(job) = self.jobs.try_recv() {
            let mut ui = Ui {
                event_loop,
                surfaces: &mut self.surfaces,
            };
            job(&mut ui);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.drain(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, (): ()) {
        self.drain(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.drain(event_loop);
    }

    fn window_event(&mut self, _: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(s) = self.surfaces.get_mut(&id) {
            s.event(&event);
        }
    }
}

/// Serves UI jobs on this (the main) thread until `done` says the
/// program has finished. Call it once, from `main`.
///
/// # Panics
///
/// When called twice in a process.
pub fn serve(done: impl Fn() -> bool) {
    let (jobs, rx) = mpsc::channel::<Job>();
    assert!(
        HOST.set(Host {
            jobs,
            wake: Mutex::new(None),
        })
        .is_ok(),
        "serve is called once"
    );
    // Wait, cheaply, for the first job: most programs never open a window.
    let first = loop {
        match rx.recv_timeout(Duration::from_millis(20)) {
            Ok(job) => break job,
            Err(RecvTimeoutError::Timeout) if !done() => {}
            Err(_) => return,
        }
    };
    let Ok(mut event_loop) = EventLoop::<()>::with_user_event().build() else {
        return;
    };
    if let Some(host) = HOST.get() {
        *host
            .wake
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(event_loop.create_proxy());
    }
    let (again, rx2) = mpsc::channel::<Job>();
    let _ = again.send(first);
    // the first job, then everything sent since, in order
    let mut app = App {
        jobs: rx2,
        surfaces: HashMap::new(),
    };
    let forward = std::thread::spawn(move || {
        while let Ok(job) = rx.recv() {
            if again.send(job).is_err() {
                break;
            }
            if let Some(host) = HOST.get() {
                if let Some(p) = host
                    .wake
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .as_ref()
                {
                    let _ = p.send_event(());
                }
            }
        }
    });
    while !done() {
        if let PumpStatus::Exit(_) =
            event_loop.pump_app_events(Some(Duration::from_millis(4)), &mut app)
        {
            break;
        }
    }
    app.surfaces.clear();
    drop(forward);
}

/// A surface's events as its program reads them: the main thread
/// pushes, the program's thread waits for the next one.
#[derive(Clone)]
pub struct Events {
    inner: Arc<(Mutex<VecDeque<String>>, Condvar)>,
    cap: usize,
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

impl Events {
    /// A window's queue: it keeps the newest 256 events, so a program
    /// that stops reading cannot make it grow without bound.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(256)
    }

    /// A queue keeping at most `cap` events.
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Arc::default(),
            cap: cap.max(1),
        }
    }

    /// Records an event (main thread), dropping the oldest when full.
    pub fn push(&self, event: impl Into<String>) {
        let (q, ready) = &*self.inner;
        let mut q = q.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if q.len() >= self.cap {
            q.pop_front();
        }
        q.push_back(event.into());
        ready.notify_all();
    }

    /// The next event, waiting at most `wait`; `"frame"` when none came
    /// (a frame tick: the program's chance to draw).
    pub fn next(&self, wait: Duration) -> String {
        let (q, ready) = &*self.inner;
        let deadline = Instant::now() + wait;
        let mut q = q.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        loop {
            if let Some(e) = q.pop_front() {
                return e;
            }
            let now = Instant::now();
            if now >= deadline {
                return "frame".into();
            }
            q = ready
                .wait_timeout(q, deadline - now)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
    }
}

/// Whether window programs run without a screen (`XETAL_HEADLESS=1`):
/// no window opens, events come from [`scripted`].
#[must_use]
pub fn headless() -> bool {
    std::env::var_os("XETAL_HEADLESS").is_some_and(|v| !v.is_empty() && v != "0")
}

/// The events of a headless window: those in `XETAL_EVENTS` (separated
/// by commas: `frame,key q`), then `close`.
#[must_use]
pub fn scripted() -> Events {
    // every scripted event is kept: a recording may script thousands
    let e = Events::with_capacity(usize::MAX);
    let script = std::env::var("XETAL_EVENTS").unwrap_or_default();
    for ev in script.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        e.push(ev);
    }
    e.push("close");
    e
}

/// Where frames are saved (`XETAL_FRAMES=DIR`), if anywhere.
#[must_use]
pub fn frames_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("XETAL_FRAMES")
        .filter(|v| !v.is_empty())
        .map(std::path::PathBuf::from)
}

/// Whether frame `n` (1, 2, ...) is saved: every frame, unless
/// `XETAL_FRAMES_ONLY` lists the ones wanted (`30,1500,3456`), so a test
/// that checks a few frames of a long run saves only those.
#[must_use]
pub fn frame_wanted(n: u64) -> bool {
    match std::env::var("XETAL_FRAMES_ONLY") {
        Ok(list) if !list.trim().is_empty() => list
            .split(',')
            .filter_map(|k| k.trim().parse::<u64>().ok())
            .any(|k| k == n),
        _ => true,
    }
}

/// Writes `width` by `height` 0RGB pixels as an RGB PNG, making the
/// directory.
///
/// # Errors
///
/// When the file cannot be written.
pub fn save_png(
    width: usize,
    height: usize,
    pixels: &[u32],
    path: &std::path::Path,
) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let w = u32::try_from(width.max(1)).map_err(|e| e.to_string())?;
    let h = u32::try_from(height.max(1)).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    #[allow(clippy::cast_possible_truncation)]
    let mut data: Vec<u8> = pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    if data.is_empty() {
        data = vec![0, 0, 0];
    }
    writer.write_image_data(&data).map_err(|e| e.to_string())
}
