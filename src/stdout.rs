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

pub struct StdoutOverrideGuard {
    stdout_fd: RawFd,
    file_fd: RawFd,
}

pub struct StdoutOverride;

impl StdoutOverride {
    pub fn override_file<P: AsRef<Path>>(p: P) -> io::Result<StdoutOverrideGuard> {
        Self::check_override();

        let file = File::create(p)?;
        let file_fd = file.into_raw_fd();
        Self::override_fd(file_fd)
    }

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
        let _ = unsafe { dup2(self.stdout_fd, STDOUT_FILENO) };
        let _ = unsafe { close(self.file_fd) };
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
