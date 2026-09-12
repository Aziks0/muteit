// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
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
            "LoadLibraryW",
            "FreeLibrary",
            "GetProcAddress",
            "RtlGetVersion",
            "OSVERSIONINFOW",
            "STATUS_SUCCESS",
        ])
        .write();
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    generate_windows_binding();
}
