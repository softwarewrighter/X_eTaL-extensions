//! scene: retained 3D line and point scenes in a native window, patched by id

use xetal_ext_sdk::{OwnedError, Value};

/// The extension's name and version, as text.
#[allow(clippy::unnecessary_wraps)] // every handler has the SDK's type
fn about(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(format!("scene {}", env!("CARGO_PKG_VERSION"))))
}

xetal_ext_sdk::xetal_extension! {
    name: "scene",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        about: 0, "Unit -> Char", "The extension's name and version.";
    }
}
