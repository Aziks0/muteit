// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use crate::windows::Win32::{EDataFlow, eAll, eCapture, eRender};

/// Audio data flow.
pub trait Flow: Clone + Copy + PartialEq + Eq {
    const DATA_FLOW: EDataFlow;
}

/// Audio input.
///
/// Data flows from the endpoint devices to the applications.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Input;
impl Flow for Input {
    const DATA_FLOW: EDataFlow = eCapture;
}

/// Audio output.
///
/// Data flows from the applications to the endpoint devices.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Output;
impl Flow for Output {
    const DATA_FLOW: EDataFlow = eRender;
}

/// Audio input and output.
///
/// Data flows from the applications to the endpoint devices.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct All;
impl Flow for All {
    const DATA_FLOW: EDataFlow = eAll;
}

/// Audio data flow.
pub enum FlowVariant {
    /// Audio input.
    ///
    /// Data flows from the endpoint devices to the applications.
    Input,
    /// Audio output.
    ///
    /// Data flows from the applications to the endpoint devices.
    Output,
}

#[allow(non_upper_case_globals)]
impl From<EDataFlow> for FlowVariant {
    fn from(value: EDataFlow) -> Self {
        match value {
            eCapture => FlowVariant::Input,
            eRender => FlowVariant::Output,
            _ => unreachable!("There are no other flow variant"),
        }
    }
}
