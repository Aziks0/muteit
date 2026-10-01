// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::env;
use std::path::Path;

use windows_bindgen::Bindgen;

fn generate_windows_binding() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("windows.rs");

    Bindgen::new()
        .output(path)
        .filters([
            "HMODULE",
            "IMAGE_DOS_HEADER",
            "HWND",
            "WPARAM",
            "LPARAM",
            "LRESULT",
            "RegisterClassW",
            "WNDCLASSW",
            "CreateWindowExW",
            "WS_EX_NOACTIVATE",
            "WS_EX_TRANSPARENT",
            "WS_EX_LAYERED",
            "WS_EX_TOOLWINDOW",
            "WS_OVERLAPPED",
            "CW_USEDEFAULT",
            "CW_USEDEFAULT",
            "DestroyWindow",
            "WM_HOTKEY",
            "DefWindowProcW",
            "MOD_NOREPEAT",
            "MOD_WIN",
            "MOD_CONTROL",
            "MOD_SHIFT",
            "MOD_ALT",
            "RegisterHotKey",
            "UnregisterHotKey",
            "ERROR_HOTKEY_ALREADY_REGISTERED",
        ])
        .write();
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    generate_windows_binding();
}
