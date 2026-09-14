// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::env;
use std::path::Path;

use minijinja::render;
use regex::RegexBuilder;
use windows_bindgen::Bindgen;

fn generate_windows_binding() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("windows.rs");

    Bindgen::new()
        .output(path)
        .filters([
            "MapVirtualKeyExW",
            "MAPVK_VSC_TO_VK",
            "MAPVK_VK_TO_CHAR",
            "VK_OEM_PLUS",
            "VK_OEM_COMMA",
            "VK_OEM_MINUS",
            "VK_OEM_PERIOD",
            "VK_OEM_1",
            "VK_OEM_2",
            "VK_OEM_3",
            "VK_OEM_4",
            "VK_OEM_5",
            "VK_OEM_6",
            "VK_OEM_7",
            "VK_BACK",
            "VK_TAB",
            "VK_SPACE",
            "VK_RETURN",
            "VK_CAPITAL",
            "VK_ESCAPE",
            "VK_PRIOR",
            "VK_NEXT",
            "VK_END",
            "VK_HOME",
            "VK_LEFT",
            "VK_UP",
            "VK_RIGHT",
            "VK_DOWN",
            "VK_SNAPSHOT",
            "VK_INSERT",
            "VK_DELETE",
            "VK_F1",
            "VK_F2",
            "VK_F3",
            "VK_F4",
            "VK_F5",
            "VK_F6",
            "VK_F7",
            "VK_F8",
            "VK_F9",
            "VK_F10",
            "VK_F11",
            "VK_F12",
            "VK_F13",
            "VK_F14",
            "VK_F15",
            "VK_F16",
            "VK_F17",
            "VK_F18",
            "VK_F19",
            "VK_F20",
            "VK_F21",
            "VK_F22",
            "VK_F23",
            "VK_F24",
            "VK_NUMLOCK",
            "VK_NUMPAD0",
            "VK_NUMPAD1",
            "VK_NUMPAD2",
            "VK_NUMPAD3",
            "VK_NUMPAD4",
            "VK_NUMPAD5",
            "VK_NUMPAD6",
            "VK_NUMPAD7",
            "VK_NUMPAD8",
            "VK_NUMPAD9",
            "VK_ADD",
            "VK_DECIMAL",
            "VK_DIVIDE",
            "VK_MULTIPLY",
            "VK_SUBTRACT",
            "VK_SCROLL",
            "VK_VOLUME_DOWN",
            "VK_VOLUME_UP",
            "VK_VOLUME_MUTE",
            "VK_PLAY",
            "VK_MEDIA_PLAY_PAUSE",
            "VK_MEDIA_STOP",
            "VK_MEDIA_NEXT_TRACK",
            "VK_MEDIA_PREV_TRACK",
            "VK_LAUNCH_MEDIA_SELECT",
            "VK_LAUNCH_MAIL",
            "VK_LAUNCH_APP1",
            "VK_LAUNCH_APP2",
            "VK_PAUSE",
            "VK_SLEEP",
            "VK_BROWSER_BACK",
            "VK_BROWSER_FORWARD",
            "VK_BROWSER_REFRESH",
            "VK_BROWSER_STOP",
            "VK_BROWSER_SEARCH",
            "VK_BROWSER_FAVORITES",
            "VK_BROWSER_HOME",
            "VK_CONTROL",
            "VK_LCONTROL",
            "VK_RCONTROL",
            "VK_SHIFT",
            "VK_LSHIFT",
            "VK_RSHIFT",
            "VK_MENU",
            "VK_LMENU",
            "VK_RMENU",
            "VK_LWIN",
            "VK_RWIN",
        ])
        .write();
}

fn main() {
    println!("cargo::rerun-if-changed=assets/dom_code_data.inc");
    println!("cargo::rerun-if-changed=assets/code_to_scancode.rs.jinja");

    generate_windows_binding();

    let data = include_str!("assets/dom_code_data.inc");

    // We want to get the Windows scancodes associated with every `KeyboardEvent.code`.
    let pattern = concat!(
        // Avoid matching commented lines.
        "^  ",
        r"DOM_CODE\(",
        // The "USB" part.
        "0x[0-9a-f]{6}",
        ", ",
        // The "evdev" and "XKB" part.
        "(?:0x[0-9a-f]{4}, ){2}",
        // The "Win" part. That's what we want.
        "(0x[0-9a-f]{4})",
        ", ",
        // The "Mac" part.
        "0x[0-9a-f]{4}",
        ", ",
        // The "Code" part. We only want the ones that are strings (i.e. not `NULL`).
        "(\"[0-9a-zA-Z]+\")",
    );
    let re = RegexBuilder::new(pattern)
        .multi_line(true)
        .build()
        .expect("regex should be built");
    let codes: Vec<_> = re
        .captures_iter(data)
        .filter_map(|caps| {
            let scancode = caps.get(1).unwrap().as_str();
            let code = caps.get(2).unwrap().as_str();

            if scancode == "0x0000" {
                None
            } else {
                Some((code, scancode))
            }
        })
        .collect();

    // Generate the file.
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("code_to_scancode.rs");
    let template = include_str!("assets/code_to_scancode.rs.jinja");
    std::fs::write(path, render!(template, codes)).expect("code_to_scancode.rs to be written");
}
