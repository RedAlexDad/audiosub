#![allow(unsafe_code)]

use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct StderrRedirect {
    saved_fd: std::os::unix::io::RawFd,
}

impl StderrRedirect {
    pub fn new(log_path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::create(log_path.as_ref())?;
        let file_fd = file.as_raw_fd();

        let saved_fd = unsafe { libc::dup(2) };
        if saved_fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let ret = unsafe { libc::dup2(file_fd, 2) };
        if ret < 0 {
            unsafe { libc::close(saved_fd) };
            return Err(io::Error::last_os_error());
        }

        Ok(StderrRedirect { saved_fd })
    }
}

impl Drop for StderrRedirect {
    fn drop(&mut self) {
        unsafe {
            libc::dup2(self.saved_fd, 2);
            libc::close(self.saved_fd);
        }
    }
}
