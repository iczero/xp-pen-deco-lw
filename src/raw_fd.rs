use std::ffi::CString;
use std::os::unix::prelude::{RawFd, AsRawFd};

use libc::{c_void, openat, read, write, AT_FDCWD, EAGAIN, O_NONBLOCK, O_RDWR};
use tokio::io::unix::AsyncFd;

use crate::util::UnixError;

/// Extremely thin async wrapper over a fd
pub struct AsyncRawFd {
    pub async_poll: AsyncFd<RawFd>,
}

impl AsyncRawFd {
    pub fn from_fd<T>(handle: &T) -> anyhow::Result<Self>
        where T: AsRawFd
    {
        Ok(AsyncRawFd {
            async_poll: AsyncFd::new(handle.as_raw_fd())?,
        })
    }

    /// Open hidraw device. Will block.
    pub fn open(path: String) -> anyhow::Result<Self> {
        unsafe {
            let fd = openat(
                AT_FDCWD,
                CString::new(path)?.into_raw(),
                O_RDWR | O_NONBLOCK,
            );
            if fd < 0 {
                Err(UnixError::capture().into())
            } else {
                Ok(AsyncRawFd {
                    async_poll: AsyncFd::new(fd)?,
                })
            }
        }
    }

    /// Get underlying fd
    pub fn fd(&self) -> RawFd {
        *self.async_poll.get_ref()
    }

    /// Asynchronous read operation
    pub async fn read(&self, length: usize) -> anyhow::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::with_capacity(length);
        unsafe {
            loop {
                // wait for readiness
                let mut ready_guard = self.async_poll.readable().await?;
                let fd = *ready_guard.get_inner();
                let read_bytes = read(fd, buf.as_mut_ptr() as *mut c_void, length);
                if read_bytes < 0 {
                    let err = UnixError::capture();
                    if err.errno == EAGAIN {
                        // fd not yet ready
                        ready_guard.clear_ready();
                        continue;
                    } else {
                        // other error
                        return Err(err.into());
                    }
                } else {
                    // read succeeded
                    buf.set_len(read_bytes as usize);
                    return Ok(buf);
                }
            }
        }
    }

    /// Asynchronous write
    pub async fn write(&self, buf: &[u8]) -> anyhow::Result<usize> {
        unsafe {
            loop {
                let mut ready_guard = self.async_poll.writable().await?;
                let fd = *ready_guard.get_inner();
                let write_bytes = write(fd, buf.as_ptr() as *const c_void, buf.len());
                if write_bytes < 0 {
                    let err = UnixError::capture();
                    if err.errno == EAGAIN {
                        // fd not readdy
                        ready_guard.clear_ready();
                        continue;
                    } else {
                        return Err(err.into());
                    }
                } else {
                    return Ok(write_bytes as usize);
                }
            }
        }
    }

    // TODO: close on drop
}
