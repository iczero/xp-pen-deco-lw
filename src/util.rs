use std::ffi::CStr;

use thiserror::Error;
use libc::{__errno_location, strerror, c_int};

pub fn errno() -> c_int {
    unsafe { *__errno_location() }
}

pub fn strerror2(errno: c_int) -> String {
    unsafe {
        let err = strerror(errno);
        let cstr = CStr::from_ptr(err);
        // Utf8Error should never occur
        String::from(cstr.to_str().unwrap())
    }
}

#[derive(Debug, Error)]
#[error("unix error {errno} ({})", strerror2(*.errno))]
pub struct UnixError {
    pub errno: c_int
}

impl UnixError {
    pub fn capture() -> Self {
        UnixError { errno: errno() }
    }
}
