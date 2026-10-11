//! blender: Blender 3D in batch, driven from X_eTaL. An X_eTaL program
//! writes the Python -- a launcher that runs Blender in the background,
//! and the bpy script Blender runs inside -- with `write`, then `run`s the
//! launcher with python3; the launcher finds Blender in `XETAL_BLENDER`.
//! The output goes to a log beside the launcher; `run` answers with the
//! exit status and the log's last lines, `files` with what was written.
//! Paths are confined to the working directory (or `XETAL_BLENDER_ROOT`),
//! as the http extension's are.

use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use xetal_ext_sdk::{OwnedError, Value, text};

/// The default time a run may take before it is stopped (15 minutes).
const TIMEOUT: Duration = Duration::from_secs(900);

/// The log lines `run` answers with.
const TAIL: usize = 20;

fn failure(message: impl Into<String>) -> OwnedError {
    OwnedError::failure(message.into())
}

/// The directory paths are under: `XETAL_BLENDER_ROOT`, or the working
/// directory.
fn root() -> Result<PathBuf, OwnedError> {
    match std::env::var_os("XETAL_BLENDER_ROOT") {
        Some(r) if !r.is_empty() => Ok(PathBuf::from(r)),
        _ => std::env::current_dir().map_err(|e| failure(format!("the working directory: {e}"))),
    }
}

/// A relative path under the root, with no `..`.
pub fn confine(path: &str) -> Result<PathBuf, OwnedError> {
    let p = Path::new(path);
    if path.is_empty()
        || p.is_absolute()
        || !p
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(OwnedError::invalid_argument(format!(
            "{path:?}: give a path under the working directory (or XETAL_BLENDER_ROOT), no .."
        )));
    }
    Ok(root()?.join(p))
}

/// Blender's program: `BLENDER`, else the macOS application, else
/// `blender` on the PATH; None when there is none.
#[must_use]
pub fn find_blender() -> Option<PathBuf> {
    if let Some(b) = std::env::var_os("BLENDER").filter(|b| !b.is_empty()) {
        return Some(PathBuf::from(b));
    }
    let app = PathBuf::from("/Applications/Blender.app/Contents/MacOS/Blender");
    if app.is_file() {
        return Some(app);
    }
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|d| d.join("blender"))
            .find(|p| p.is_file())
    })
}

/// What a run came to: the exit status (None: stopped at the timeout) and
/// everything it printed.
pub struct Ran {
    pub status: Option<i32>,
    pub output: String,
}

/// Runs `program args` in `dir`, with `env` added, stopping it after
/// `timeout`; its output (standard output and error, in one stream).
pub fn run_with(
    program: &str,
    args: &[String],
    dir: &Path,
    env: &[(&str, String)],
    timeout: Duration,
) -> Result<Ran, OwnedError> {
    let log = tempfile_in(dir)?;
    let out =
        std::fs::File::create(&log).map_err(|e| failure(format!("{}: {e}", log.display())))?;
    let err = out
        .try_clone()
        .map_err(|e| failure(format!("{}: {e}", log.display())))?;
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err));
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| failure(format!("{program}: {e}")))?;
    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().map_err(|e| failure(e.to_string()))? {
            break s.code();
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let mut output = String::new();
    std::fs::File::open(&log)
        .and_then(|mut f| f.read_to_string(&mut output))
        .map_err(|e| failure(format!("{}: {e}", log.display())))?;
    let _ = std::fs::remove_file(&log);
    Ok(Ran { status, output })
}

/// A fresh file name in `dir` for a run's output while it runs: the
/// process and a count, so runs at the same time never share one.
fn tempfile_in(dir: &Path) -> Result<PathBuf, OwnedError> {
    static COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let k = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let name = format!(".blender-run-{}-{k}.out", std::process::id());
    std::fs::create_dir_all(dir).map_err(|e| failure(format!("{}: {e}", dir.display())))?;
    Ok(dir.join(name))
}

/// The last `n` lines of `text`.
#[must_use]
pub fn tail(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

/// The extension's name and version, as text.
#[allow(clippy::unnecessary_wraps)] // every handler has the SDK's type
fn about(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(format!(
        "blender {}",
        env!("CARGO_PKG_VERSION")
    )))
}

/// path write text: the text into the file (made whole, or not at all),
/// its directory made; how many bytes.
fn write(args: &[Value]) -> Result<Value, OwnedError> {
    let name = text(&args[0])?;
    let body = text(&args[1])?;
    let path = confine(&name)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| failure(format!("{name}: {e}")))?;
    }
    let part = path.with_extension("part");
    std::fs::write(&part, body.as_bytes()).map_err(|e| failure(format!("{name}: {e}")))?;
    std::fs::rename(&part, &path).map_err(|e| failure(format!("{name}: {e}")))?;
    Ok(Value::Int(i64::try_from(body.len()).unwrap_or(i64::MAX)))
}

/// path read: the file's text (a small file a run wrote: a report).
fn read(args: &[Value]) -> Result<Value, OwnedError> {
    let name = text(&args[0])?;
    let path = confine(&name)?;
    std::fs::read_to_string(&path)
        .map(Value::Text)
        .map_err(|e| failure(format!("{name}: {e}")))
}

/// blender: the Blender program found, or "" when there is none.
#[allow(clippy::unnecessary_wraps)]
fn blender(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(
        find_blender()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
    ))
}

/// version: the first line Blender prints for --version, or "".
fn version(_: &[Value]) -> Result<Value, OwnedError> {
    let Some(b) = find_blender() else {
        return Ok(Value::Text(String::new()));
    };
    let dir = std::env::temp_dir();
    let ran = run_with(
        &b.display().to_string(),
        &["--version".into()],
        &dir,
        &[],
        Duration::from_secs(60),
    )?;
    Ok(Value::Text(
        ran.output
            .lines()
            .find(|l| l.starts_with("Blender"))
            .unwrap_or("")
            .trim()
            .to_string(),
    ))
}

/// script run args: `python3 script args` (args split at spaces) from the
/// root, with `XETAL_BLENDER` set to the Blender found, stopped after
/// `XETAL_BLENDER_TIMEOUT` seconds (900 by default). The whole output goes
/// to `script.log`; the answer is "exit N" (or "timeout after S s") and
/// the log's last 20 lines.
fn run(args: &[Value]) -> Result<Value, OwnedError> {
    let script = text(&args[0])?;
    let words = text(&args[1])?;
    let path = confine(&script)?;
    let dir = root()?;
    let mut argv = vec![path.display().to_string()];
    argv.extend(words.split_whitespace().map(String::from));
    let python = std::env::var("XETAL_PYTHON").unwrap_or_else(|_| "python3".into());
    let timeout = std::env::var("XETAL_BLENDER_TIMEOUT")
        .ok()
        .and_then(|t| t.parse().ok())
        .map_or(TIMEOUT, Duration::from_secs);
    let blender = find_blender()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let ran = run_with(&python, &argv, &dir, &[("XETAL_BLENDER", blender)], timeout)?;
    let log = path.with_extension("log");
    std::fs::write(&log, ran.output.as_bytes())
        .map_err(|e| failure(format!("{}: {e}", log.display())))?;
    let head = match ran.status {
        Some(code) => format!("exit {code}"),
        None => format!("timeout after {} s", timeout.as_secs()),
    };
    Ok(Value::Text(format!("{head}\n{}", tail(&ran.output, TAIL))))
}

/// files dir: the files under dir, one a line, "bytes path", sorted.
fn files(args: &[Value]) -> Result<Value, OwnedError> {
    let name = text(&args[0])?;
    let top = confine(&name)?;
    let mut found = Vec::new();
    let mut todo = vec![top.clone()];
    while let Some(d) = todo.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                todo.push(p);
            } else if let Ok(m) = e.metadata() {
                let rel = p.strip_prefix(&top).unwrap_or(&p).display().to_string();
                found.push(format!("{} {name}/{rel}", m.len()));
            }
        }
    }
    found.sort_by(|a, b| {
        a.split_once(' ')
            .map(|x| x.1)
            .cmp(&b.split_once(' ').map(|x| x.1))
    });
    Ok(Value::Text(found.join("\n")))
}

xetal_ext_sdk::xetal_extension! {
    name: "blender",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        about: 0, "Unit -> Char", "The extension's name and version.";
        write: 2, "Char -> Char -> Int", "path write text: the text into the file, made whole; how many bytes.";
        blender: 0, "Unit -> Char", "The Blender program found (BLENDER, the macOS app, PATH), or \"\".";
        version: 0, "Unit -> Char", "Blender's version line, or \"\".";
        run: 2, "Char -> Char -> Char", "script run args: python3 script args, XETAL_BLENDER set; exit N and the log's last lines.";
        read: 1, "Char -> Char", "path read: the file's text.";
        files: 1, "Char -> Char", "files dir: the files written under dir, \"bytes path\" a line.";
    }
}
