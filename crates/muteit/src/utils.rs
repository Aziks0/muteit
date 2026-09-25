// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::LazyLock;

use anyhow::Context;
use iced::keyboard::key;
use iced::window::settings::PlatformSpecific;
use iced::window::settings::platform::IconType;
use iced::{Size, Task, window};

use audio::interface::audio_controller::AudioController;

use crate::windows::Win32::{GetSystemMetrics, SM_CXICON, SM_CXSMICON};

pub type DeviceEnumerator<F> = Rc<RefCell<audio::enumerator::DeviceEnumerator<F>>>;

#[inline]
pub fn toggle_mute(controller: &Option<AudioController>) -> anyhow::Result<()> {
    if let Some(controller) = controller {
        // TODO: replace by `try` block when stabilized.
        // https://doc.rust-lang.org/std/ops/trait.Try.html
        let inner = || -> anyhow::Result<()> {
            let muted = controller.get_mute()?;
            controller.set_mute(!muted)?;

            Ok(())
        };
        inner().context("Toggling mute state.")?;
    }

    Ok(())
}

/// Get icon from the resource's icons using the system's expected dimensions.
fn get_icon(metric: i32) -> IconType {
    let size = unsafe { GetSystemMetrics(metric) } as u32;
    assert!(size != 0, "Could not get system metrics.");

    let size = Size::new(size, size);

    IconType::Resource {
        ordinal: 1,
        size: Some(size),
    }
}

#[inline(always)]
fn get_small_icon() -> IconType {
    get_icon(SM_CXSMICON)
}

#[inline(always)]
fn get_big_icon() -> IconType {
    get_icon(SM_CXICON)
}

pub fn open_window<T>() -> (window::Id, Task<T>)
where
    T: Send + 'static,
{
    // Static so we don't call `utils::get_[size]_icon` everytime.
    static DEFAULT_WINDOW_SETTINGS: LazyLock<window::Settings> =
        LazyLock::new(|| window::Settings {
            size: Size::new(800., 230.),
            platform_specific: PlatformSpecific {
                drag_and_drop: false,
                icon: Some(get_small_icon()),
                taskbar_icon: Some(get_big_icon()),
                ..Default::default()
            },
            ..Default::default()
        });

    let (id, task) = window::open(DEFAULT_WINDOW_SETTINGS.clone());

    (id, task.discard())
}

#[cold]
pub fn leak_error(error: impl Into<anyhow::Error>) -> &'static anyhow::Error {
    let error = Box::new(error.into());
    Box::leak(error)
}

/// Translate `iced`'s [`Modifiers`](iced::keyboard::Modifiers) into `hotkey`'s
/// [`Modifiers`](hotkey::Modifiers).
pub const fn translate_modifiers(iced_modifiers: iced::keyboard::Modifiers) -> hotkey::Modifiers {
    use hotkey::Modifiers as HotKeyModifiers;
    use iced::keyboard::Modifiers as IcedModifiers;

    let mut mods = HotKeyModifiers::NONE;

    if iced_modifiers.contains(IcedModifiers::LOGO) {
        mods = mods.union(HotKeyModifiers::WIN);
    };
    if iced_modifiers.contains(IcedModifiers::CTRL) {
        mods = mods.union(HotKeyModifiers::CTRL);
    };
    if iced_modifiers.contains(IcedModifiers::SHIFT) {
        mods = mods.union(HotKeyModifiers::SHIFT);
    };
    if iced_modifiers.contains(IcedModifiers::ALT) {
        mods = mods.union(HotKeyModifiers::ALT);
    };

    mods
}

/// Get the `str` version of [`key::Named`].
pub const fn named_key_as_str(name: key::Named) -> &'static str {
    match name {
        key::Named::Alt => "Alt",
        key::Named::AltGraph => "AltGraph",
        key::Named::CapsLock => "CapsLock",
        key::Named::Control => "Control",
        key::Named::Fn => "Fn",
        key::Named::FnLock => "FnLock",
        key::Named::NumLock => "NumLock",
        key::Named::ScrollLock => "ScrollLock",
        key::Named::Shift => "Shift",
        key::Named::Symbol => "Symbol",
        key::Named::SymbolLock => "SymbolLock",
        key::Named::Meta => "Meta",
        key::Named::Hyper => "Hyper",
        key::Named::Super => "Super",
        key::Named::Enter => "Enter",
        key::Named::Tab => "Tab",
        key::Named::Space => "Space",
        key::Named::ArrowDown => "ArrowDown",
        key::Named::ArrowLeft => "ArrowLeft",
        key::Named::ArrowRight => "ArrowRight",
        key::Named::ArrowUp => "ArrowUp",
        key::Named::End => "End",
        key::Named::Home => "Home",
        key::Named::PageDown => "PageDown",
        key::Named::PageUp => "PageUp",
        key::Named::Backspace => "Backspace",
        key::Named::Clear => "Clear",
        key::Named::Copy => "Copy",
        key::Named::CrSel => "CrSel",
        key::Named::Cut => "Cut",
        key::Named::Delete => "Delete",
        key::Named::EraseEof => "EraseEof",
        key::Named::ExSel => "ExSel",
        key::Named::Insert => "Insert",
        key::Named::Paste => "Paste",
        key::Named::Redo => "Redo",
        key::Named::Undo => "Undo",
        key::Named::Accept => "Accept",
        key::Named::Again => "Again",
        key::Named::Attn => "Attn",
        key::Named::Cancel => "Cancel",
        key::Named::ContextMenu => "ContextMenu",
        key::Named::Escape => "Escape",
        key::Named::Execute => "Execute",
        key::Named::Find => "Find",
        key::Named::Help => "Help",
        key::Named::Pause => "Pause",
        key::Named::Play => "Play",
        key::Named::Props => "Props",
        key::Named::Select => "Select",
        key::Named::ZoomIn => "ZoomIn",
        key::Named::ZoomOut => "ZoomOut",
        key::Named::BrightnessDown => "BrightnessDown",
        key::Named::BrightnessUp => "BrightnessUp",
        key::Named::Eject => "Eject",
        key::Named::LogOff => "LogOff",
        key::Named::Power => "Power",
        key::Named::PowerOff => "PowerOff",
        key::Named::PrintScreen => "PrintScreen",
        key::Named::Hibernate => "Hibernate",
        key::Named::Standby => "Standby",
        key::Named::WakeUp => "WakeUp",
        key::Named::AllCandidates => "AllCandidates",
        key::Named::Alphanumeric => "Alphanumeric",
        key::Named::CodeInput => "CodeInput",
        key::Named::Compose => "Compose",
        key::Named::Convert => "Convert",
        key::Named::FinalMode => "FinalMode",
        key::Named::GroupFirst => "GroupFirst",
        key::Named::GroupLast => "GroupLast",
        key::Named::GroupNext => "GroupNext",
        key::Named::GroupPrevious => "GroupPrevious",
        key::Named::ModeChange => "ModeChange",
        key::Named::NextCandidate => "NextCandidate",
        key::Named::NonConvert => "NonConvert",
        key::Named::PreviousCandidate => "PreviousCandidate",
        key::Named::Process => "Process",
        key::Named::SingleCandidate => "SingleCandidate",
        key::Named::HangulMode => "HangulMode",
        key::Named::HanjaMode => "HanjaMode",
        key::Named::JunjaMode => "JunjaMode",
        key::Named::Eisu => "Eisu",
        key::Named::Hankaku => "Hankaku",
        key::Named::Hiragana => "Hiragana",
        key::Named::HiraganaKatakana => "HiraganaKatakana",
        key::Named::KanaMode => "KanaMode",
        key::Named::KanjiMode => "KanjiMode",
        key::Named::Katakana => "Katakana",
        key::Named::Romaji => "Romaji",
        key::Named::Zenkaku => "Zenkaku",
        key::Named::ZenkakuHankaku => "ZenkakuHankaku",
        key::Named::Soft1 => "Soft1",
        key::Named::Soft2 => "Soft2",
        key::Named::Soft3 => "Soft3",
        key::Named::Soft4 => "Soft4",
        key::Named::ChannelDown => "ChannelDown",
        key::Named::ChannelUp => "ChannelUp",
        key::Named::Close => "Close",
        key::Named::MailForward => "MailForward",
        key::Named::MailReply => "MailReply",
        key::Named::MailSend => "MailSend",
        key::Named::MediaClose => "MediaClose",
        key::Named::MediaFastForward => "MediaFastForward",
        key::Named::MediaPause => "MediaPause",
        key::Named::MediaPlay => "MediaPlay",
        key::Named::MediaPlayPause => "MediaPlayPause",
        key::Named::MediaRecord => "MediaRecord",
        key::Named::MediaRewind => "MediaRewind",
        key::Named::MediaStop => "MediaStop",
        key::Named::MediaTrackNext => "MediaTrackNext",
        key::Named::MediaTrackPrevious => "MediaTrackPrevious",
        key::Named::New => "New",
        key::Named::Open => "Open",
        key::Named::Print => "Print",
        key::Named::Save => "Save",
        key::Named::SpellCheck => "SpellCheck",
        key::Named::Key11 => "Key11",
        key::Named::Key12 => "Key12",
        key::Named::AudioBalanceLeft => "AudioBalanceLeft",
        key::Named::AudioBalanceRight => "AudioBalanceRight",
        key::Named::AudioBassBoostDown => "AudioBassBoostDown",
        key::Named::AudioBassBoostToggle => "AudioBassBoostToggle",
        key::Named::AudioBassBoostUp => "AudioBassBoostUp",
        key::Named::AudioFaderFront => "AudioFaderFront",
        key::Named::AudioFaderRear => "AudioFaderRear",
        key::Named::AudioSurroundModeNext => "AudioSurroundModeNext",
        key::Named::AudioTrebleDown => "AudioTrebleDown",
        key::Named::AudioTrebleUp => "AudioTrebleUp",
        key::Named::AudioVolumeDown => "AudioVolumeDown",
        key::Named::AudioVolumeUp => "AudioVolumeUp",
        key::Named::AudioVolumeMute => "AudioVolumeMute",
        key::Named::MicrophoneToggle => "MicrophoneToggle",
        key::Named::MicrophoneVolumeDown => "MicrophoneVolumeDown",
        key::Named::MicrophoneVolumeUp => "MicrophoneVolumeUp",
        key::Named::MicrophoneVolumeMute => "MicrophoneVolumeMute",
        key::Named::SpeechCorrectionList => "SpeechCorrectionList",
        key::Named::SpeechInputToggle => "SpeechInputToggle",
        key::Named::LaunchApplication1 => "LaunchApplication1",
        key::Named::LaunchApplication2 => "LaunchApplication2",
        key::Named::LaunchCalendar => "LaunchCalendar",
        key::Named::LaunchContacts => "LaunchContacts",
        key::Named::LaunchMail => "LaunchMail",
        key::Named::LaunchMediaPlayer => "LaunchMediaPlayer",
        key::Named::LaunchMusicPlayer => "LaunchMusicPlayer",
        key::Named::LaunchPhone => "LaunchPhone",
        key::Named::LaunchScreenSaver => "LaunchScreenSaver",
        key::Named::LaunchSpreadsheet => "LaunchSpreadsheet",
        key::Named::LaunchWebBrowser => "LaunchWebBrowser",
        key::Named::LaunchWebCam => "LaunchWebCam",
        key::Named::LaunchWordProcessor => "LaunchWordProcessor",
        key::Named::BrowserBack => "BrowserBack",
        key::Named::BrowserFavorites => "BrowserFavorites",
        key::Named::BrowserForward => "BrowserForward",
        key::Named::BrowserHome => "BrowserHome",
        key::Named::BrowserRefresh => "BrowserRefresh",
        key::Named::BrowserSearch => "BrowserSearch",
        key::Named::BrowserStop => "BrowserStop",
        key::Named::AppSwitch => "AppSwitch",
        key::Named::Call => "Call",
        key::Named::Camera => "Camera",
        key::Named::CameraFocus => "CameraFocus",
        key::Named::EndCall => "EndCall",
        key::Named::GoBack => "GoBack",
        key::Named::GoHome => "GoHome",
        key::Named::HeadsetHook => "HeadsetHook",
        key::Named::LastNumberRedial => "LastNumberRedial",
        key::Named::Notification => "Notification",
        key::Named::MannerMode => "MannerMode",
        key::Named::VoiceDial => "VoiceDial",
        key::Named::TV => "TV",
        key::Named::TV3DMode => "TV3DMode",
        key::Named::TVAntennaCable => "TVAntennaCable",
        key::Named::TVAudioDescription => "TVAudioDescription",
        key::Named::TVAudioDescriptionMixDown => "TVAudioDescriptionMixDown",
        key::Named::TVAudioDescriptionMixUp => "TVAudioDescriptionMixUp",
        key::Named::TVContentsMenu => "TVContentsMenu",
        key::Named::TVDataService => "TVDataService",
        key::Named::TVInput => "TVInput",
        key::Named::TVInputComponent1 => "TVInputComponent1",
        key::Named::TVInputComponent2 => "TVInputComponent2",
        key::Named::TVInputComposite1 => "TVInputComposite1",
        key::Named::TVInputComposite2 => "TVInputComposite2",
        key::Named::TVInputHDMI1 => "TVInputHDMI1",
        key::Named::TVInputHDMI2 => "TVInputHDMI2",
        key::Named::TVInputHDMI3 => "TVInputHDMI3",
        key::Named::TVInputHDMI4 => "TVInputHDMI4",
        key::Named::TVInputVGA1 => "TVInputVGA1",
        key::Named::TVMediaContext => "TVMediaContext",
        key::Named::TVNetwork => "TVNetwork",
        key::Named::TVNumberEntry => "TVNumberEntry",
        key::Named::TVPower => "TVPower",
        key::Named::TVRadioService => "TVRadioService",
        key::Named::TVSatellite => "TVSatellite",
        key::Named::TVSatelliteBS => "TVSatelliteBS",
        key::Named::TVSatelliteCS => "TVSatelliteCS",
        key::Named::TVSatelliteToggle => "TVSatelliteToggle",
        key::Named::TVTerrestrialAnalog => "TVTerrestrialAnalog",
        key::Named::TVTerrestrialDigital => "TVTerrestrialDigital",
        key::Named::TVTimer => "TVTimer",
        key::Named::AVRInput => "AVRInput",
        key::Named::AVRPower => "AVRPower",
        key::Named::ColorF0Red => "ColorF0Red",
        key::Named::ColorF1Green => "ColorF1Green",
        key::Named::ColorF2Yellow => "ColorF2Yellow",
        key::Named::ColorF3Blue => "ColorF3Blue",
        key::Named::ColorF4Grey => "ColorF4Grey",
        key::Named::ColorF5Brown => "ColorF5Brown",
        key::Named::ClosedCaptionToggle => "ClosedCaptionToggle",
        key::Named::Dimmer => "Dimmer",
        key::Named::DisplaySwap => "DisplaySwap",
        key::Named::DVR => "DVR",
        key::Named::Exit => "Exit",
        key::Named::FavoriteClear0 => "FavoriteClear0",
        key::Named::FavoriteClear1 => "FavoriteClear1",
        key::Named::FavoriteClear2 => "FavoriteClear2",
        key::Named::FavoriteClear3 => "FavoriteClear3",
        key::Named::FavoriteRecall0 => "FavoriteRecall0",
        key::Named::FavoriteRecall1 => "FavoriteRecall1",
        key::Named::FavoriteRecall2 => "FavoriteRecall2",
        key::Named::FavoriteRecall3 => "FavoriteRecall3",
        key::Named::FavoriteStore0 => "FavoriteStore0",
        key::Named::FavoriteStore1 => "FavoriteStore1",
        key::Named::FavoriteStore2 => "FavoriteStore2",
        key::Named::FavoriteStore3 => "FavoriteStore3",
        key::Named::Guide => "Guide",
        key::Named::GuideNextDay => "GuideNextDay",
        key::Named::GuidePreviousDay => "GuidePreviousDay",
        key::Named::Info => "Info",
        key::Named::InstantReplay => "InstantReplay",
        key::Named::Link => "Link",
        key::Named::ListProgram => "ListProgram",
        key::Named::LiveContent => "LiveContent",
        key::Named::Lock => "Lock",
        key::Named::MediaApps => "MediaApps",
        key::Named::MediaAudioTrack => "MediaAudioTrack",
        key::Named::MediaLast => "MediaLast",
        key::Named::MediaSkipBackward => "MediaSkipBackward",
        key::Named::MediaSkipForward => "MediaSkipForward",
        key::Named::MediaStepBackward => "MediaStepBackward",
        key::Named::MediaStepForward => "MediaStepForward",
        key::Named::MediaTopMenu => "MediaTopMenu",
        key::Named::NavigateIn => "NavigateIn",
        key::Named::NavigateNext => "NavigateNext",
        key::Named::NavigateOut => "NavigateOut",
        key::Named::NavigatePrevious => "NavigatePrevious",
        key::Named::NextFavoriteChannel => "NextFavoriteChannel",
        key::Named::NextUserProfile => "NextUserProfile",
        key::Named::OnDemand => "OnDemand",
        key::Named::Pairing => "Pairing",
        key::Named::PinPDown => "PinPDown",
        key::Named::PinPMove => "PinPMove",
        key::Named::PinPToggle => "PinPToggle",
        key::Named::PinPUp => "PinPUp",
        key::Named::PlaySpeedDown => "PlaySpeedDown",
        key::Named::PlaySpeedReset => "PlaySpeedReset",
        key::Named::PlaySpeedUp => "PlaySpeedUp",
        key::Named::RandomToggle => "RandomToggle",
        key::Named::RcLowBattery => "RcLowBattery",
        key::Named::RecordSpeedNext => "RecordSpeedNext",
        key::Named::RfBypass => "RfBypass",
        key::Named::ScanChannelsToggle => "ScanChannelsToggle",
        key::Named::ScreenModeNext => "ScreenModeNext",
        key::Named::Settings => "Settings",
        key::Named::SplitScreenToggle => "SplitScreenToggle",
        key::Named::STBInput => "STBInput",
        key::Named::STBPower => "STBPower",
        key::Named::Subtitle => "Subtitle",
        key::Named::Teletext => "Teletext",
        key::Named::VideoModeNext => "VideoModeNext",
        key::Named::Wink => "Wink",
        key::Named::ZoomToggle => "ZoomToggle",
        key::Named::F1 => "F1",
        key::Named::F2 => "F2",
        key::Named::F3 => "F3",
        key::Named::F4 => "F4",
        key::Named::F5 => "F5",
        key::Named::F6 => "F6",
        key::Named::F7 => "F7",
        key::Named::F8 => "F8",
        key::Named::F9 => "F9",
        key::Named::F10 => "F10",
        key::Named::F11 => "F11",
        key::Named::F12 => "F12",
        key::Named::F13 => "F13",
        key::Named::F14 => "F14",
        key::Named::F15 => "F15",
        key::Named::F16 => "F16",
        key::Named::F17 => "F17",
        key::Named::F18 => "F18",
        key::Named::F19 => "F19",
        key::Named::F20 => "F20",
        key::Named::F21 => "F21",
        key::Named::F22 => "F22",
        key::Named::F23 => "F23",
        key::Named::F24 => "F24",
        key::Named::F25 => "F25",
        key::Named::F26 => "F26",
        key::Named::F27 => "F27",
        key::Named::F28 => "F28",
        key::Named::F29 => "F29",
        key::Named::F30 => "F30",
        key::Named::F31 => "F31",
        key::Named::F32 => "F32",
        key::Named::F33 => "F33",
        key::Named::F34 => "F34",
        key::Named::F35 => "F35",
    }
}
