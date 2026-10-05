//! What a macro may know besides its call (MC20, MC24): configuration
//! facts for `[]C_FG` (the platform, `cli` or `web`, and the flags given
//! with `xetal --cfg NAME`) and files read by `[]I_NCLUDE`.

use std::sync::Mutex;

use xetal_base::Diagnostic;

static FLAGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The flags `xetal --cfg NAME` sets, for `[]C_FG`.
pub fn set_flags(flags: Vec<String>) {
    *FLAGS.lock().unwrap_or_else(|e| e.into_inner()) = flags;
}

/// Whether `name` holds: the platform this runs on (`cli`, or `web` in
/// the browser) or a flag that was set.
pub(crate) fn holds(name: &str) -> bool {
    let platform = if cfg!(target_arch = "wasm32") {
        "web"
    } else {
        "cli"
    };
    name == platform
        || FLAGS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .any(|f| f == name)
}

/// The text of `path`, relative to the directory of `file` (the file
/// the call is written in): from the disk at the command line, from the
/// store in the browser. An absolute path is refused, so a program
/// names only files beside it (MC24, proposed).
pub(crate) fn include(path: &str, file: &str) -> Result<String, Diagnostic> {
    let place = |d: Diagnostic| d.with_note("macro-place: right");
    if path.starts_with('/') || path.starts_with('\\') || path.contains(':') {
        let message =
            format!("i_nclude< reads a file by a path relative to the program, not {path:?}");
        return Err(place(Diagnostic::new("bad-include", message)));
    }
    let dir = std::path::Path::new(file)
        .parent()
        .filter(|p| !p.as_os_str().is_empty() && file != "-e");
    let full = dir.map_or(path.to_string(), |d| d.join(path).display().to_string());
    xetal_store::read(&full).map_err(|_| {
        let message = format!("there is no file {full:?} to include");
        place(Diagnostic::new("missing-file", message))
    })
}
