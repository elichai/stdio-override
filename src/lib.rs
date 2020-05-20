#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]
#![allow(clippy::match_wild_err_arm)]

//! # Stdio-Override
//!
//! This crate provides a library for overriding Stdio file descriptors. <br>
//! It provides a Guard for the replacement so that when the guard is dropped the file descriptors are switched back
//! and the replacement File Descriptor will be closed.
//!
//! **Trying to replace an std File Descriptor twice without dropping the guard will result in a panic** <br>
//!
//! **Notice:** When trying to use this in tests you **must** run with `cargo test -- --nocapture` otherwise it will redirect stdout/stderr again.
//!
//! This library is made to be intuitive and easy to use.
//!
//! ## Examples
//! Stdout:
//! ```rust
//!# fn main() -> std::io:: Result<()> {
//! use stdio_override::StdoutOverride;
//! use std::fs;
//! let file_name = "./test.txt";
//!# std::fs::remove_file(file_name);
//! let guard = StdoutOverride::override_file(file_name)?;
//! println!("Isan to Stdout!");
//!
//! let contents = fs::read_to_string(file_name)?;
//! assert_eq!("Isan to Stdout!\n", contents);
//! drop(guard);
//! println!("Outside!");
//!
//!# Ok(())
//!# }
//! ```
//!
//! Stderr:
//! ```rust
//! # fn main() -> std::io:: Result<()> {
//! use stdio_override::StderrOverride;
//! use std::fs;
//! let file_name = "./testerr.txt";
//! # std::fs::remove_file(file_name);
//! let guard = StderrOverride::override_file(file_name)?;
//! eprintln!("Failure to stderr");
//!
//! let contents = fs::read_to_string(file_name)?;
//! assert_eq!("Failure to stderr\n", contents);
//! drop(guard);
//! eprintln!("Stderr is back!");
//!
//! # Ok(())
//! # }
//! ```
//!
//! Stdin:
//! ```rust
//! # fn main() -> std::io:: Result<()> {
//! # use std::{fs::{self, File},io::{self, Write}};
//! use stdio_override::StdinOverride;
//! let file_name = "./inputs.txt";
//! # std::fs::remove_file(file_name);
//!
//! {
//!     let mut file = File::create(&file_name)?;
//!     file.write_all(b"Inputs to stdin")?;
//! }
//! let guard = StdinOverride::override_file(file_name)?;
//! let mut user_input = String::new();
//! io::stdin().read_line(&mut user_input)?;
//!
//! drop(guard);
//!
//! assert_eq!("Inputs to stdin", user_input);
//! // Stdin is working as usual again, because the guard is dropped.
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
        drop(guard);

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
        drop(guard);

        let contents = read_to_string(file_path)?;
        assert_eq!(data, contents);
        eprintln!("Outside!");

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

        drop(guard);

        assert_eq!(data, s);

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
