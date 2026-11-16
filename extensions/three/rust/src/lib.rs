//! three: interactive 3D in a browser. An X_eTaL program writes a scene
//! as the calls of a player (the calls Cycles.xtl writes for Blender);
//! here they go to a player written with Three.js, which draws them in
//! real time; a second player draws a game's board (view, camera,
//! tiles, piece). In a browser (wasm32) `show` hands the calls to the page
//! this code is linked into; natively `page` writes a directory to
//! serve: the page, the players, Three.js and the calls.
//!
//! Paths are confined to the working directory (or `XETAL_THREE_ROOT`),
//! as the blender extension's are.

use xetal_ext_sdk::{OwnedError, Value, text};

/// The player: the functions a scene's calls use, drawn with Three.js.
pub const PLAYER: &str = include_str!("../../web/static/player.js");
/// The board player: a game's board (view, camera, tiles, piece).
pub const BOARD: &str = include_str!("../../web/static/board.js");
/// The board player's modules, a group of calls each: the path under
/// the page, the text.
pub const BOARD_PARTS: &[(&str, &str)] = &[
    (
        "board/light.js",
        include_str!("../../web/static/board/light.js"),
    ),
    (
        "board/terrain.js",
        include_str!("../../web/static/board/terrain.js"),
    ),
    (
        "board/terrain-textures.js",
        include_str!("../../web/static/board/terrain-textures.js"),
    ),
    (
        "board/water.js",
        include_str!("../../web/static/board/water.js"),
    ),
    (
        "board/particles.js",
        include_str!("../../web/static/board/particles.js"),
    ),
    (
        "board/eye.js",
        include_str!("../../web/static/board/eye.js"),
    ),
];
/// A game's page side: the clock, keys and clicks of a program kept
/// running, its pictures handed to the board player.
pub const GAME: &str = include_str!("../../web/static/game.js");
/// The page's controls: riding, flying, walking, and showing a scene.
pub const LIVE: &str = include_str!("../../web/static/live.js");
/// The page `page` writes, which reads the calls from scene.calls.js.
pub const INDEX: &str = include_str!("../../web/static/standalone.html");

/// Three.js 0.170.0 (MIT), the passes the player uses and the glTF
/// loader the board player reads its pieces with, as vendored
/// in web/static/vendor/three: the path under the page, the text.
pub const VENDOR: &[(&str, &str)] = &[
    (
        "vendor/three/LICENSE",
        include_str!("../../web/static/vendor/three/LICENSE"),
    ),
    (
        "vendor/three/three.module.min.js",
        include_str!("../../web/static/vendor/three/three.module.min.js"),
    ),
    (
        "vendor/three/loaders/GLTFLoader.js",
        include_str!("../../web/static/vendor/three/loaders/GLTFLoader.js"),
    ),
    (
        "vendor/three/utils/BufferGeometryUtils.js",
        include_str!("../../web/static/vendor/three/utils/BufferGeometryUtils.js"),
    ),
    (
        "vendor/three/postprocessing/EffectComposer.js",
        include_str!("../../web/static/vendor/three/postprocessing/EffectComposer.js"),
    ),
    (
        "vendor/three/postprocessing/MaskPass.js",
        include_str!("../../web/static/vendor/three/postprocessing/MaskPass.js"),
    ),
    (
        "vendor/three/postprocessing/OutputPass.js",
        include_str!("../../web/static/vendor/three/postprocessing/OutputPass.js"),
    ),
    (
        "vendor/three/postprocessing/Pass.js",
        include_str!("../../web/static/vendor/three/postprocessing/Pass.js"),
    ),
    (
        "vendor/three/postprocessing/RenderPass.js",
        include_str!("../../web/static/vendor/three/postprocessing/RenderPass.js"),
    ),
    (
        "vendor/three/postprocessing/ShaderPass.js",
        include_str!("../../web/static/vendor/three/postprocessing/ShaderPass.js"),
    ),
    (
        "vendor/three/postprocessing/UnrealBloomPass.js",
        include_str!("../../web/static/vendor/three/postprocessing/UnrealBloomPass.js"),
    ),
    (
        "vendor/three/shaders/CopyShader.js",
        include_str!("../../web/static/vendor/three/shaders/CopyShader.js"),
    ),
    (
        "vendor/three/shaders/LuminosityHighPassShader.js",
        include_str!("../../web/static/vendor/three/shaders/LuminosityHighPassShader.js"),
    ),
    (
        "vendor/three/shaders/OutputShader.js",
        include_str!("../../web/static/vendor/three/shaders/OutputShader.js"),
    ),
];

/// How many calls a scene's text holds: its lines that are not empty.
pub fn calls(scene: &str) -> i64 {
    i64::try_from(scene.lines().filter(|l| !l.trim().is_empty()).count()).unwrap_or(i64::MAX)
}

#[cfg(not(target_arch = "wasm32"))]
mod place {
    use std::path::{Component, Path, PathBuf};

    use xetal_ext_sdk::OwnedError;

    use super::{BOARD, BOARD_PARTS, GAME, INDEX, LIVE, PLAYER, VENDOR};

    fn failure(message: impl Into<String>) -> OwnedError {
        OwnedError::failure(message)
    }

    /// The directory paths are under: `XETAL_THREE_ROOT`, or the working
    /// directory.
    fn root() -> Result<PathBuf, OwnedError> {
        match std::env::var_os("XETAL_THREE_ROOT") {
            Some(r) if !r.is_empty() => Ok(PathBuf::from(r)),
            _ => {
                std::env::current_dir().map_err(|e| failure(format!("the working directory: {e}")))
            }
        }
    }

    /// The path under the root: relative, with no `..`.
    ///
    /// # Errors
    ///
    /// InvalidArgument for an absolute path or one that climbs out.
    pub fn confine(path: &str) -> Result<PathBuf, OwnedError> {
        let p = Path::new(path);
        let inside = !path.is_empty()
            && p.components()
                .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
        if !inside {
            return Err(OwnedError::invalid_argument(format!(
                "{path:?}: give a path under the working directory (or XETAL_THREE_ROOT), no .."
            )));
        }
        Ok(root()?.join(p))
    }

    fn put(dir: &Path, name: &str, body: &str) -> Result<(), OwnedError> {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| failure(format!("{name}: {e}")))?;
        }
        std::fs::write(&path, body.as_bytes()).map_err(|e| failure(format!("{name}: {e}")))
    }

    /// Writes the page, the players, Three.js and the scene's calls into
    /// the directory (made); how many files.
    pub fn page(dir: &str, scene: &str) -> Result<i64, OwnedError> {
        let dir = confine(dir)?;
        put(&dir, "index.html", INDEX)?;
        put(&dir, "player.js", PLAYER)?;
        put(&dir, "board.js", BOARD)?;
        put(&dir, "game.js", GAME)?;
        put(&dir, "live.js", LIVE)?;
        put(&dir, "scene.calls.js", scene)?;
        for (name, body) in BOARD_PARTS.iter().chain(VENDOR) {
            put(&dir, name, body)?;
        }
        Ok(i64::try_from(6 + BOARD_PARTS.len() + VENDOR.len()).unwrap_or(i64::MAX))
    }

    /// There is no page to draw in: nothing to do.
    #[allow(clippy::unnecessary_wraps)] // the browser's can fail
    pub fn show(_scene: &str) -> Result<(), OwnedError> {
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod place {
    use wasm_bindgen::{JsCast, JsValue};
    use xetal_ext_sdk::OwnedError;

    fn failure(message: impl Into<String>) -> OwnedError {
        OwnedError::failure(message)
    }

    /// A browser has no files to write.
    pub fn page(_dir: &str, _scene: &str) -> Result<i64, OwnedError> {
        Err(failure(
            "th:p_age! writes files: in a browser use th:s_how!",
        ))
    }

    /// Hands the calls to the page: `xetalThree.show(calls)`, which the
    /// page's live.js defines.
    pub fn show(scene: &str) -> Result<(), OwnedError> {
        let global = js_sys::global();
        let three = js_sys::Reflect::get(&global, &JsValue::from_str("xetalThree"))
            .map_err(|_| failure("this page has no xetalThree"))?;
        let show = js_sys::Reflect::get(&three, &JsValue::from_str("show"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .ok_or_else(|| failure("this page has no player (xetalThree.show)"))?;
        show.call1(&three, &JsValue::from_str(scene))
            .map(|_| ())
            .map_err(|e| failure(format!("the player: {}", describe(&e))))
    }

    fn describe(e: &JsValue) -> String {
        e.dyn_ref::<js_sys::Error>()
            .map(|e| String::from(e.message()))
            .or_else(|| e.as_string())
            .unwrap_or_else(|| "an error".to_string())
    }
}

/// The extension's name and version, as text.
#[allow(clippy::unnecessary_wraps)] // every handler has the SDK's type
fn about(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(format!("three {}", env!("CARGO_PKG_VERSION"))))
}

/// Draws a scene in the page (a browser); natively there is no page
/// and nothing is drawn. How many calls the scene holds.
fn show(args: &[Value]) -> Result<Value, OwnedError> {
    let scene = text(&args[0])?;
    place::show(&scene)?;
    Ok(Value::Int(calls(&scene)))
}

/// Writes a directory to serve: the page, the players, Three.js and the
/// scene's calls. How many files.
fn page(args: &[Value]) -> Result<Value, OwnedError> {
    let dir = text(&args[0])?;
    let scene = text(&args[1])?;
    place::page(&dir, &scene).map(Value::Int)
}

xetal_ext_sdk::xetal_extension! {
    name: "three",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        about: 0, "Unit -> Char", "The extension's name and version.";
        show: 1, "Char -> Int", "Draw a scene (the calls of a player) in the page; how many calls. Natively nothing is drawn.";
        page: 2, "Char Char -> Int", "Write a directory to serve -- the page, the players, Three.js, the scene's calls; how many files.";
    }
}
