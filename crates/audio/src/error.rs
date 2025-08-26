use windows::Win32::Media::Audio::AUDCLNT_E_DEVICE_INVALIDATED;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid volume level (expected value between 0.0 and 1.0, found {0})")]
    InvalidVolume(f32),
    #[error(
        "the audio device has been unplugged, or the audio hardware has been \
        reconfigured, disabled, remove, or otherwise made unavailable for use"
    )]
    DeviceInvalidated,
    #[error("unknown windows error ({0})")]
    UnknownWindows(windows::core::Error),
}

impl From<windows::core::Error> for Error {
    fn from(error: windows::core::Error) -> Self {
        match error.code() {
            AUDCLNT_E_DEVICE_INVALIDATED => Self::DeviceInvalidated,
            _ => Self::UnknownWindows(error),
        }
    }
}
