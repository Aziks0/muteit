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
            "ERROR_NOT_FOUND",
            "ERROR_DEVICE_REMOVED",
            "ERROR_NO_SUCH_DEVICE_INTERFACE",
            "E_INVALIDARG",
            "CO_E_NOTINITIALIZED",
            "AUDCLNT_E_DEVICE_INVALIDATED",
            "AUDIO_VOLUME_NOTIFICATION_DATA",
            "IAudioEndpointVolume::{GetMute, SetMute, RegisterControlChangeNotify, UnregisterControlChangeNotify}",
            "IAudioEndpointVolumeCallback",
            "IMMNotificationClient",
            "IMMDevice",
            "IMMDeviceEnumerator",
            "IMMDeviceCollection",
            "MMDeviceEnumerator",
            "DEVICE_STATE_ACTIVE",
            "DEVICE_STATE_DISABLED",
            "DEVICE_STATE_NOTPRESENT",
            "DEVICE_STATE_UNPLUGGED",
            "ERole",
            "eCommunications",
            "eConsole",
            "eMultimedia",
            "EDataFlow",
            "eAll",
            "eCapture",
            "eRender",
            "CoCreateInstance",
            "CoTaskMemFree",
            "CLSCTX_ALL",
            "PKEY_Device_FriendlyName",
            "IPropertyStore",
            "STGM_READ",
            "PropVariantToBSTR",
        ])
        .write();
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    generate_windows_binding();
}
