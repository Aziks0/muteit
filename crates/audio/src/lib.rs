#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod device;
use device::Device;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
