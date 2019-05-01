use crate::ffi::{close, dup, dup2, STDOUT_FILENO};

use std::fs::File;
use std::io;
use std::os::unix::io::IntoRawFd;
use std::os::unix::io::RawFd;
use std::os::unix::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_REPLACED: AtomicBool = AtomicBool::new(false);

const ORDERING: Ordering = Ordering::SeqCst;

/// A Guard over the Stdout change.
/// when this guard is dropped stdout will go back to the original,
/// and the file will be closed.
pub struct StdoutOverrideGuard {
    stdout_fd: RawFd,
    file_fd: RawFd,
}

/// Override the Stdout File Descriptor safely.
///
pub struct StdoutOverride;

impl StdoutOverride {
    /// Override the stdout by providing a path.
    /// This uses [`File::create`] so it will fail/succeed accordingly.
    ///
    /// [`File::create`]: https://doc.rust-lang.org/stable/std/fs/struct.File.html#method.create
    pub fn override_file<P: AsRef<Path>>(p: P) -> io::Result<StdoutOverrideGuard> {
        Self::check_override();

        let file = File::create(p)?;
        let file_fd = file.into_raw_fd();
        Self::override_fd(file_fd)
    }

    /// Override the stdout by providing something that can be turned into a file descriptor.
    /// This will accept Sockets, Files, and even Stdio's. [`AsRawFd`]
    ///
    /// [`AsRawFd`]: https://doc.rust-lang.org/stable/std/os/unix/io/trait.AsRawFd.html
    pub fn override_raw<FD: AsRawFd>(fd: FD) -> io::Result<StdoutOverrideGuard> {
        Self::check_override();

        let file_fd = fd.as_raw_fd();
        Self::override_fd(file_fd)
    }

    fn override_fd(file_fd: RawFd) -> io::Result<StdoutOverrideGuard> {
        let stdout_fd = unsafe { dup(STDOUT_FILENO) }?;
        let _ = unsafe { dup2(file_fd, STDOUT_FILENO) }?;

        IS_REPLACED.store(true, ORDERING);

        Ok(StdoutOverrideGuard { stdout_fd, file_fd })
    }

    fn check_override() {
        if IS_REPLACED.load(ORDERING) {
            panic!("Tried to override Stdout twice");
        }
    }
}

impl Drop for StdoutOverrideGuard {
    fn drop(&mut self) {
        // Ignoring syscalls errors seems to be the most sensible thing to do in a Drop impl
        // https://github.com/rust-lang/rust/blob/bd177f3e/src/libstd/sys/unix/fd.rs#L293-L302
        let _ = unsafe { dup2(self.stdout_fd, STDOUT_FILENO) };
        let _ = unsafe { close(self.file_fd) };
        IS_REPLACED.store(false, ORDERING);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs::{File, remove_file}, io::{Read, Write, stdout}, mem};
    #[test]
    fn test_stdout() {
        let file_name = "./test.txt";
        let data = "12345";
        let _ = remove_file(file_name);

        let guard = StdoutOverride::override_file(file_name).unwrap();
        print!("{}", data);
        stdout().flush().unwrap();
        mem::drop(guard);

        let mut file = File::open("test.txt").unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(data, contents);
        println!("Outside!");

        remove_file(file_name).unwrap();
    }
}
