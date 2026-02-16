// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use crate::enumerator::DeviceInformation;

pub trait AudioInterface: for<'a> TryFrom<&'a DeviceInformation> {
    type Interface: windows_core::Interface;

    fn new(interface: Self::Interface) -> Self;
}
