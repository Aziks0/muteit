// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    #[cfg(feature = "__test")]
    println!("cargo::rustc-env=APP_ID=436a1306-cf3a-4b24-960f-dd1fe948463b");
}
