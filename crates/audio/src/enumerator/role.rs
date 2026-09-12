// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use crate::windows::Win32::{ERole, eCommunications, eConsole, eMultimedia};

#[derive(Clone, Copy, PartialEq, Eq)]
/// Device role assigned by the system or the user.
pub enum Role {
    /// Games, system notification sounds, and voice commands.
    Console,
    /// Music, movies, narration, and live music recording.
    Multimedia,
    /// Voice communications (talking to another person).
    Communications,
}

#[allow(non_upper_case_globals)]
impl From<ERole> for Role {
    fn from(value: ERole) -> Self {
        match value {
            eConsole => Self::Console,
            eMultimedia => Self::Multimedia,
            eCommunications => Self::Communications,
            _ => unreachable!("There are no other device role."),
        }
    }
}
