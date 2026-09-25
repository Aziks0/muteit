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
        .filters(["MapVirtualKeyExW", "MAPVK_VSC_TO_VK"])
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
        "\"([0-9a-zA-Z]+)\"",
    );
    let re = RegexBuilder::new(pattern)
        .multi_line(true)
        .build()
        .expect("regex should be built");
    let mut codes: Vec<_> = re
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

    // `Meta` has been renamed to `Super` by `winit` for some reason.
    for (code, _) in &mut codes {
        match *code {
            "MetaLeft" => *code = "SuperLeft",
            "MetaRight" => *code = "SuperRight",
            _ => {}
        }
    }

    // Generate the file.
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("code_to_scancode.rs");
    let template = include_str!("assets/code_to_scancode.rs.jinja");
    std::fs::write(path, render!(template, codes)).expect("code_to_scancode.rs to be written");
}
