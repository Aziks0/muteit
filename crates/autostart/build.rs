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
            "ERROR_SUCCESS",
            "ERROR_FILE_NOT_FOUND",
            "ERROR_MORE_DATA",
            "RegDeleteKeyValueW",
            "RegGetValueW",
            "RegSetKeyValueW",
            "HKEY_CURRENT_USER",
            "REG_SZ",
            "RRF_RT_REG_SZ",
        ])
        .write();
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    #[cfg(feature = "__test")]
    println!("cargo::rustc-env=CARGO_BIN_NAME=autostart");

    generate_windows_binding();
}
