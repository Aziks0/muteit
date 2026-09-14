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
        .filters(["CreateMutexW", "ERROR_ALREADY_EXISTS"])
        .write();
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    #[cfg(feature = "__test")]
    println!("cargo::rustc-env=APP_ID=436a1306-cf3a-4b24-960f-dd1fe948463b");

    generate_windows_binding();
}
