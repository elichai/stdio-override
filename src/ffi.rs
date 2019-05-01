use libc::{self, c_int};
pub use libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use std::io::{Error, Result};

pub unsafe fn dup(fd: c_int) -> Result<c_int> {
    let res = libc::dup(fd);
    if res == -1 {
        Err(Error::last_os_error())
    } else {
        Ok(res)
    }
}

pub unsafe fn dup2(src: c_int, dst: c_int) -> Result<c_int> {
    let res = libc::dup2(src, dst);
    if res == -1 {
        Err(Error::last_os_error())
    } else {
        Ok(res)
    }
}

pub unsafe fn close(fd: c_int) -> Result<()> {
    let res = libc::close(fd);
    if res == -1 {
        Err(Error::last_os_error())
    } else {
        // res == 0 is success, nothing else is returned.
        Ok(())
    }
}
