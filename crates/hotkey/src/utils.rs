// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::windows::Win32::{HINSTANCE, HMODULE, IMAGE_DOS_HEADER};

// SPDX-SnippetBegin
//
// SPDX-SnippetCopyrightText: 2026 winit contributors <https://github.com/rust-windowing/winit>
// SPDX-SnippetCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: Apache-2.0
//
// SPDX-SnippetComment: Initially taken from the `winit` git repository
// <https://github.com/rust-windowing/winit.git>, revision
// `8b5f46d4be7d29f5b0ed91f4343b78d8c2668bd0`, in file `winit/winit-win32/src/util.rs`, line
// 148–161.

/// Get this instance module handle.
pub const fn get_instance_handle() -> HMODULE {
    // Gets the instance handle by taking the address of the
    // pseudo-variable created by the microsoft linker:
    // https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483

    // This is preferred over GetModuleHandle(NULL) because it also works in DLLs:
    // https://stackoverflow.com/questions/21718027/getmodulehandlenull-vs-hinstance

    unsafe extern "C" {
        static mut __ImageBase: IMAGE_DOS_HEADER;
    }

    let image_base_ptr = &raw mut __ImageBase as *mut _;

    HINSTANCE(image_base_ptr as _)
}

// SPDX-SnippetEnd
