//! digest: SHA-256 and CRC-32 of text and of files, for checking that
//! content is what it should be (a fetched feed against its recorded
//! digest). Files are named by paths under the working directory (or
//! `XETAL_DIGEST_ROOT`) and read in pieces, so their size does not
//! matter.

use std::fmt::Write as _;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use xetal_ext_sdk::{OwnedError, Value, text};

fn failure(m: impl Into<String>) -> OwnedError {
    OwnedError::failure(m)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// A path a program names, under the root: relative, with no `..`.
fn confine(path: &str) -> Result<PathBuf, OwnedError> {
    let p = Path::new(path);
    if path.is_empty()
        || p.is_absolute()
        || !p
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(OwnedError::invalid_argument(format!(
            "{path:?}: give a path under the working directory (or XETAL_DIGEST_ROOT), no .."
        )));
    }
    let root = match std::env::var_os("XETAL_DIGEST_ROOT") {
        Some(r) => PathBuf::from(r),
        None => std::env::current_dir().map_err(|e| failure(e.to_string()))?,
    };
    Ok(root.join(p))
}

/// Every piece of a file, in order.
fn each_piece(path: &str, mut f: impl FnMut(&[u8])) -> Result<(), OwnedError> {
    let full = confine(path)?;
    let mut file = std::fs::File::open(&full).map_err(|e| failure(format!("{path}: {e}")))?;
    let mut buf = vec![0; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| failure(format!("{path}: {e}")))?;
        if n == 0 {
            return Ok(());
        }
        f(&buf[..n]);
    }
}

/// sha256 text: its SHA-256 (of its UTF-8 bytes), 64 hex digits.
fn sha256(args: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(hex(&Sha256::digest(
        text(&args[0])?.as_bytes(),
    ))))
}

/// sha256_file path: the file's SHA-256, 64 hex digits.
fn sha256_file(args: &[Value]) -> Result<Value, OwnedError> {
    let mut h = Sha256::new();
    each_piece(&text(&args[0])?, |b| h.update(b))?;
    Ok(Value::Text(hex(&h.finalize())))
}

/// crc32 text: its CRC-32 (IEEE, as zip and PNG use), a number.
fn crc32(args: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Int(i64::from(crc32fast::hash(
        text(&args[0])?.as_bytes(),
    ))))
}

/// crc32_file path: the file's CRC-32.
fn crc32_file(args: &[Value]) -> Result<Value, OwnedError> {
    let mut h = crc32fast::Hasher::new();
    each_piece(&text(&args[0])?, |b| h.update(b))?;
    Ok(Value::Int(i64::from(h.finalize())))
}

xetal_ext_sdk::xetal_extension! {
    name: "digest",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        sha256: 1, "Char -> Char", "The SHA-256 of a text (its UTF-8 bytes), as 64 hex digits.";
        sha256_file: 1, "Char -> Char", "The SHA-256 of a file, as 64 hex digits.";
        crc32: 1, "Char -> Int", "The CRC-32 (IEEE) of a text.";
        crc32_file: 1, "Char -> Int", "The CRC-32 (IEEE) of a file.";
    }
}
