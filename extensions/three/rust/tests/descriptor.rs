//! three's descriptor validates and lists its functions.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::validate_descriptor;

#[test]
fn descriptor() {
    let ext =
        unsafe { validate_descriptor(xetal_ext_three::__xetal_extension::descriptor()) }.unwrap();
    assert_eq!(ext.name(), "three");
    assert_eq!(ext.function("about").unwrap().signature(), "Unit -> Char");
    assert_eq!(ext.function("show").unwrap().signature(), "Char -> Int");
    assert_eq!(
        ext.function("page").unwrap().signature(),
        "Char Char -> Int"
    );
}

#[test]
fn a_scene_is_counted_by_its_calls() {
    assert_eq!(xetal_ext_three::calls(""), 0);
    assert_eq!(xetal_ext_three::calls("stage(1, 2)\n\nreport()\n"), 2);
}

#[test]
fn the_player_defines_every_call_the_vocabulary_writes() {
    // the calls Cycles.xtl writes: each must be a function of the player
    for call in [
        "stage", "look", "sky", "bloom", "surface", "neon", "facade", "wet", "screen", "reel",
        "boxes", "sign", "model", "fleet", "lamps", "spots", "fog", "steam", "fly", "still",
        "frames", "encode", "report", "nums", "media", "slides",
    ] {
        assert!(
            xetal_ext_three::PLAYER.contains(&format!("    {call}(")),
            "the player has no {call}"
        );
    }
}

/// A module of the board player, by its path under the page.
fn part(path: &str) -> &'static str {
    xetal_ext_three::BOARD_PARTS
        .iter()
        .find(|(p, _)| *p == path)
        .map_or("", |(_, body)| *body)
}

#[test]
fn the_board_player_defines_its_calls() {
    // the calls a game's view writes: view, camera and piece are
    // functions of the board player, tiles is kept by its line
    for call in ["view", "camera", "piece"] {
        assert!(
            xetal_ext_three::BOARD.contains(&format!("    {call}(")),
            "the board player has no {call}"
        );
    }
    assert!(xetal_ext_three::BOARD.contains("const tiles = ("));
    assert!(xetal_ext_three::BOARD.contains("const kept = { tiles }"));
    // its modules are whole, and the player names each
    assert_eq!(xetal_ext_three::BOARD_PARTS.len(), 6);
    for name in ["light", "terrain", "water", "particles", "eye"] {
        let path = format!("board/{name}.js");
        let body = part(&path);
        assert!(body.contains("export default function"), "{path}");
        assert!(
            xetal_ext_three::BOARD.contains(&format!("'./{path}'")),
            "{path}"
        );
    }
    assert!(xetal_ext_three::BOARD.contains("const nums = ("));
    // the calls its modules draw, each a function of its module
    assert!(part("board/eye.js").contains("each: { eye: call }"));
    assert!(part("board/terrain.js").contains("    terrain(line, name, corners"));
    assert!(part("board/terrain.js").contains("    ground(line, name, "));
    assert!(part("board/terrain.js").contains("'./terrain-textures.js'"));
    for call in ["sun", "sky", "fog"] {
        assert!(
            part("board/light.js").contains(&format!("    {call}(")),
            "board/light.js has no {call}"
        );
    }
    assert!(part("board/water.js").contains("water("));
    assert!(part("board/particles.js").contains("particles("));
    // the page chooses the player by the scene's first call
    assert!(xetal_ext_three::LIVE.contains("isBoard(calls)"));
}

#[test]
fn the_vendored_three_is_whole() {
    assert_eq!(xetal_ext_three::VENDOR.len(), 14);
    assert!(
        xetal_ext_three::VENDOR
            .iter()
            .all(|(_, body)| !body.is_empty())
    );
    assert!(xetal_ext_three::VENDOR[0].1.contains("MIT License"));
}
