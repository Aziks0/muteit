// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    #[cfg(feature = "__test")]
    println!("cargo::rustc-env=CARGO_BIN_NAME=autostart");
}
