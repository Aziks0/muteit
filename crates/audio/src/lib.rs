#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");
