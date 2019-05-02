#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! # Stdio-Override
//!
//! This crate provides a library for overriding Stdio file descriptors. <br>
//! It provides a Guard for the replacement so that when the guard is dropped the file descriptors are switched back
//! and the replacement File Descriptor will be closed.
//!
//! ** Trying to replace an std File Descriptor twice will result in a panic ** <br>
//!
//! *Notice:* When trying to use this in tests you **must** run with `cargo test -- --nocapture` otherwise it will redirect stdout/stderr again.
//!
//! This library is made to be intuitive and easy to use.
//!
//! ## Examples
//! ```rust
//!# fn main() -> std::io:: Result<()> {
//! use stdio_override::StdoutOverride;
//! use std::{fs, mem};
//! let file_name = "./test.txt";
//!# std::fs::remove_file(file_name);
//! let guard = StdoutOverride::override_file(file_name)?;
//! println!("Isan to Stdout!");
//!
//! let contents = fs::read_to_string(file_name)?;
//! assert_eq!("Isan to Stdout!\n", contents);
//! mem::drop(guard);
//! println!("Outside!");
//!
//!# Ok(())
//!# }
//! ```
//!
mod ffi;
#[macro_use]
mod macros;

fd_guard!(StdoutOverride, guard: StdoutOverrideGuard, FD: crate::ffi::STDOUT_FILENO, name of FD: stdout);
fd_guard!(StdinOverride, guard: StdinOverrideGuard, FD: crate::ffi::STDIN_FILENO, name of FD: stdin);
fd_guard!(StderrOverride, guard: StderrOverrideGuard, FD: crate::ffi::STDERR_FILENO, name of FD: stderr);

pub use crate::stderr::{StderrOverride, StderrOverrideGuard};
pub use crate::stdin::{StdinOverride, StdinOverrideGuard};
pub use crate::stdout::{StdoutOverride, StdoutOverrideGuard};

#[cfg(feature = "test-readme")]
doc_comment::doctest!("../README.md");

#[cfg(test)]
mod test {
    use crate::ffi::*;
    use crate::*;
    use std::os::unix::io::IntoRawFd;
    use std::{
        fs::{read_to_string, File},
        io::{stdin, stdout, Read, Result, Write},
        mem,
    };
    use tempfile;

    fn get_new_file_path() -> Result<tempfile::TempPath> {
        Ok(tempfile::NamedTempFile::new()?.into_temp_path())
    }

    #[test]
    fn test_stdout() -> Result<()> {
        let file_path = get_new_file_path()?;
        let data = "12345";

        let guard = StdoutOverride::override_file(&file_path)?;
        print!("{}", data);
        stdout().flush()?;
        mem::drop(guard);

        let contents = read_to_string(file_path)?;
        assert_eq!(data, contents);
        println!("Outside!");

        Ok(())
    }

    #[test]
    fn test_stderr() -> Result<()> {
        let file_path = get_new_file_path()?;
        let data = "123456";

        let guard = StderrOverride::override_file(&file_path)?;
        eprint!("{}", data);
        stdout().flush()?;
        mem::drop(guard);

        let contents = read_to_string(file_path)?;
        assert_eq!(data, contents);
        println!("Outside!");

        Ok(())
    }

    #[test]
    fn test_stdin() -> Result<()> {
        let file_path = get_new_file_path()?;
        let data = "12345";
        {
            let mut file = File::create(&file_path)?;
            file.write_all(data.as_bytes())?;
        }

        let guard = StdinOverride::override_file(&file_path)?;

        let mut s = String::new();
        print!("Please enter some text: ");
        stdout().flush()?;
        stdin().read_line(&mut s)?;

        mem::drop(guard);

        let contents = read_to_string(file_path)?;
        assert_eq!(data, contents);

        println!("You typed: {}", s);

        Ok(())
    }

    #[test]
    fn test_original() -> Result<()> {
        let file_path = get_new_file_path()?;

        let file = File::create(&file_path)?;
        let file = file.into_raw_fd();

        let real_stdout = unsafe { dup(STDOUT_FILENO) }?;

        unsafe { dup2(file, STDOUT_FILENO) }?;

        println!("Let's see where it's saved");
        let mut file = File::open(&file_path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        stdout().lock().flush()?;
        unsafe { dup2(real_stdout, STDOUT_FILENO) }?;
        assert_eq!("Let\'s see where it\'s saved\n", contents);

        println!("got back");

        Ok(())
    }
}
