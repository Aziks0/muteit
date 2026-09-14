// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: CC0-1.0

#![allow(non_snake_case, clippy::upper_case_acronyms)]

include!(concat!(env!("OUT_DIR"), "/windows.rs"));

pub use Windows::*;

// FIXME: Those are impossible to get with bindgen.
// https://github.com/microsoft/windows-rs/issues/4922
pub const VK_0: i32 = 0x30;
pub const VK_1: i32 = 0x31;
pub const VK_2: i32 = 0x32;
pub const VK_3: i32 = 0x33;
pub const VK_4: i32 = 0x34;
pub const VK_5: i32 = 0x35;
pub const VK_6: i32 = 0x36;
pub const VK_7: i32 = 0x37;
pub const VK_8: i32 = 0x38;
pub const VK_9: i32 = 0x39;
pub const VK_A: i32 = 0x41;
pub const VK_B: i32 = 0x42;
pub const VK_C: i32 = 0x43;
pub const VK_D: i32 = 0x44;
pub const VK_E: i32 = 0x45;
pub const VK_F: i32 = 0x46;
pub const VK_G: i32 = 0x47;
pub const VK_H: i32 = 0x48;
pub const VK_I: i32 = 0x49;
pub const VK_J: i32 = 0x4A;
pub const VK_K: i32 = 0x4B;
pub const VK_L: i32 = 0x4C;
pub const VK_M: i32 = 0x4D;
pub const VK_N: i32 = 0x4E;
pub const VK_O: i32 = 0x4F;
pub const VK_P: i32 = 0x50;
pub const VK_Q: i32 = 0x51;
pub const VK_R: i32 = 0x52;
pub const VK_S: i32 = 0x53;
pub const VK_T: i32 = 0x54;
pub const VK_U: i32 = 0x55;
pub const VK_V: i32 = 0x56;
pub const VK_W: i32 = 0x57;
pub const VK_X: i32 = 0x58;
pub const VK_Y: i32 = 0x59;
pub const VK_Z: i32 = 0x5A;
