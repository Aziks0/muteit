//! Zero-cost abstraction COM library initialization and auto-magical uninitialization.
//!
//! See the [`microsoft documentation`] for more information on COM library.
//!
//! # Example
//!
//! ```ignore
//! use coinit::{CO, APARTMENT_THREADED};
//!
//! // We can't just use `_` because Rust will drop the value instantly!
//! let _com = CO::<APARTMENT_THREADED>::init();
//!
//! // Do things with the COM library...
//!
//! // and when `_com` goes out of scope, the COM library will get uninitialized.
//! ```
//!
//! [`microsoft documentation`]:
//!     https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coinitializeex

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

use windows::Win32::Foundation::{S_FALSE, S_OK};
use windows::Win32::System::Com::{COINIT, CoInitializeEx, CoUninitialize};

/// Multithread apartment, equivalent to `COINIT_MULTITHREADED`.
pub const MULTI_THREADED: i32 = 0;
/// Single-thread apartment, equivalent to `COINIT_APARTMENTTHREADED`.
pub const APARTMENT_THREADED: i32 = 2;

/// Zero-cost abstraction COM library initialization and auto-magical uninitialization.
///
/// Valid models are [`MULTI_THREADED`] and [`APARTMENT_THREADED`].
///
/// # Example
///
/// ```ignore
/// // We can't just use `_` because Rust will drop the value instantly!
/// let _com = CO::<APARTMENT_THREADED>::init();
///
/// // Do things with the COM library...
///
/// // and when `_com` goes out of scope, the COM library will get uninitialized.
/// ```
pub struct CO<const MODEL: i32>;

impl<const MODEL: i32> EnsureValidModel for CO<MODEL> {
    const RESULT: () = assert!(
        MODEL == MULTI_THREADED || MODEL == APARTMENT_THREADED,
        "model should either be `MULTI_THREADED` or `APARTMENT_THREADED`"
    );
}

impl<const MODEL: i32> CO<MODEL> {
    /// Initialize the COM library.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // We can't just use `_` because Rust will drop the value instantly!
    /// let _com = CO::<APARTMENT_THREADED>::init();
    /// ```
    ///
    /// # Panics
    ///
    /// This will panic if the COM library has already been initialized with a different model.
    pub fn init() -> Self {
        // Compile-time assert that we got a correct model. This has zero runtime overhead.
        let _ = <CO<MODEL> as EnsureValidModel>::RESULT;

        match unsafe { CoInitializeEx(None, COINIT(MODEL)) } {
            S_OK => {}
            S_FALSE => {} // `S_FALSE` still requires us to call `CoUninitialize`.
            result => {
                let model_name = if MODEL == MULTI_THREADED {
                    "MULTI_THREADED"
                } else {
                    "APARTMENT_THREADED"
                };
                panic!(
                    "failed to initialize COM with model {} (HRESULT({}): {})",
                    model_name,
                    result.0,
                    result.message()
                )
            }
        };

        CO {}
    }
}

impl<const MODEL: i32> Drop for CO<MODEL> {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}

/// Just a trait with a const because we need it to make our compile-time assertion to ensure we use
/// a correct model.
trait EnsureValidModel {
    const RESULT: ();
}

#[cfg(test)]
mod test {
    use super::*;
    use windows::Win32::Foundation::CO_E_NOTINITIALIZED;
    use windows::Win32::System::Com::IContext;

    #[test]
    fn works() {
        let _com = CO::<APARTMENT_THREADED>::init();

        // We should be able to call a COM library function without a `CO_E_NOTINITIALIZED` error.
        use windows::Win32::System::Com::CoGetObjectContext;
        let result = unsafe { CoGetObjectContext::<IContext>() }
            .is_err_and(|err| err.code() == CO_E_NOTINITIALIZED);
        assert!(!result, "COM has not been initialized");
    }

    #[test]
    fn uninitialization() {
        {
            let _com = CO::<APARTMENT_THREADED>::init();
        }

        use windows::Win32::System::Com::CoGetObjectContext;
        let result = unsafe { CoGetObjectContext::<IContext>() }
            .is_err_and(|err| err.code() == CO_E_NOTINITIALIZED);
        assert!(result, "COM has not been uninitialized");
    }
}

// Really shitty way to test for failed compilation.
/// ```compile_fail
/// use coinit::CO;
/// CO::<30>::init();
/// ```
#[doc(hidden)]
#[allow(dead_code)]
fn compile_fail() {}
