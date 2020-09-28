#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! # Stdio-Override
//!
//! This crate provides a library for overriding Stdio file descriptors. <br>
//! It provides a guard for the replacement so that when the guard is dropped the file descriptors are switched back
//! and the replacement File Descriptor will be closed.
//!
//! You can replace a standard stream twice, just keep in mind that each guard, when dropped, will
//! replace stdout with the stdout that existed when it was created. This means that if you don't
//! drop the guards in reverse order of their creation you won't end up back where you started.
//!
//! You can use the [`os_pipe`](https://docs.rs/os_pipe) crate to capture the standard streams in
//! memory.
//!
//! **Notice:** When trying to use this in tests you **must** run with `cargo test -- --test-threads=1 --nocapture` otherwise it will redirect stdout/stderr again.
//!
//! This library is made to be intuitive and easy to use.
//!
//! ## Examples
//! Stdout:
//! ```rust
//!# fn main() -> std::io::Result<()> {
//! use stdio_override::StdoutOverride;
//! use std::fs;
//! let file_name = "./test.txt";
//!# std::fs::remove_file(file_name);
//! let guard = StdoutOverride::from_file(file_name)?;
//!
//! println!("Isan to Stdout!");
//! let contents = fs::read_to_string(file_name)?;
//! assert_eq!("Isan to Stdout!\n", contents);
//!
//! drop(guard);
//! println!("Outside!");
//!
//!# Ok(())
//!# }
//! ```
//!
//! Stderr:
//! ```rust
//! # fn main() -> std::io::Result<()> {
//! use stdio_override::StderrOverride;
//! use std::fs;
//! let file_name = "./testerr.txt";
//! # std::fs::remove_file(file_name);
//! let guard = StderrOverride::from_file(file_name)?;
//!
//! eprintln!("Failure to stderr");
//! let contents = fs::read_to_string(file_name)?;
//! assert_eq!("Failure to stderr\n", contents);
//!
//! drop(guard);
//! eprintln!("Stderr is back!");
//!
//! # Ok(())
//! # }
//! ```
//!
//! Stdin:
//! ```rust
//! # fn main() -> std::io::Result<()> {
//! # use std::{fs::{self, File}, io::{self, Write}};
//! use stdio_override::StdinOverride;
//! let file_name = "./inputs.txt";
//! fs::write(file_name, "Inputs to stdin")?;
//!
//! let guard = StdinOverride::from_file(file_name)?;
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

use std::fs::File;
use std::io::{self, IoSlice, IoSliceMut, Read, Write};
use std::mem::ManuallyDrop;
use std::path::Path;

#[cfg(not(any(unix)))]
compile_error!("stdio-override only supports Unix");

#[cfg_attr(unix, path = "unix.rs")]
mod imp;

/// An overridden standard input.
///
/// Reading from this reads the original standard input. When it is dropped the standard input
/// will be reset.
#[derive(Debug)]
pub struct StdinOverride {
    original: ManuallyDrop<File>,
    reset: bool,
}
impl StdinOverride {
    /// Read standard input from the raw file descriptor. The file descriptor must be readable.
    ///
    /// The file descriptor is not owned, so it is your job to close it later. Closing it while
    /// this exists will not close the standard error.
    pub fn from_raw(raw: imp::Raw) -> io::Result<Self> {
        Ok(Self { original: ManuallyDrop::new(imp::override_stdin(raw, false)?), reset: false })
    }
    /// Read standard input from the owned raw file descriptor. The file descriptor must be
    /// readable.
    ///
    /// The file descriptor is owned, and so you must not use it after passing it to this function.
    pub fn from_raw_owned(raw: imp::Raw) -> io::Result<Self> {
        Ok(Self { original: ManuallyDrop::new(imp::override_stdin(raw, true)?), reset: false })
    }
    /// Read standard input from the IO device. The device must be readable.
    ///
    /// Dropping the IO device after calling this function will not close the standard input.
    pub fn from_io_ref<T: imp::AsRaw>(io: &T) -> io::Result<Self> {
        Self::from_raw(imp::as_raw(io))
    }
    /// Read standard input from the IO device. The device must be readable.
    pub fn from_io<T: imp::IntoRaw>(io: T) -> io::Result<Self> {
        Self::from_raw_owned(imp::into_raw(io))
    }
    /// Read standard input from the file at that file path.
    ///
    /// The file must exist and be readable.
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::from_io(File::open(path)?)
    }
    /// Reset the standard input to its state before this type was constructed.
    ///
    /// This can be called to manually handle errors produced by the destructor.
    pub fn reset(mut self) -> io::Result<()> {
        self.reset_inner()?;
        self.reset = true;
        Ok(())
    }
    fn reset_inner(&self) -> io::Result<()> {
        imp::reset_stdin(imp::as_raw(&*self.original))
    }
}
impl Read for StdinOverride {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.original.read(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        self.original.read_vectored(bufs)
    }
}
impl<'a> Read for &'a StdinOverride {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&*self.original).read(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        (&*self.original).read_vectored(bufs)
    }
}
impl Drop for StdinOverride {
    fn drop(&mut self) {
        if !self.reset {
            let _ = self.reset_inner();
        }
    }
}

/// An overridden standard output.
///
/// Writing to this writes to the original standard output. When it is dropped the standard output
/// will be reset.
#[derive(Debug)]
pub struct StdoutOverride {
    original: ManuallyDrop<File>,
    reset: bool,
}
impl StdoutOverride {
    /// Redirect standard output to the raw file descriptor. The file descriptor must be writable.
    ///
    /// The file descriptor is not owned, so it is your job to close it later. Closing it while
    /// this exists will not close the standard output.
    pub fn from_raw(raw: imp::Raw) -> io::Result<Self> {
        Ok(Self { original: ManuallyDrop::new(imp::override_stdout(raw, false)?), reset: false })
    }
    /// Redirect standard output to the owned raw file descriptor. The file descriptor must be
    /// writable.
    ///
    /// The file descriptor is owned, and so you must not use it after passing it to this function.
    pub fn from_raw_owned(raw: imp::Raw) -> io::Result<Self> {
        Ok(Self { original: ManuallyDrop::new(imp::override_stdout(raw, true)?), reset: false })
    }
    /// Redirect standard output to the IO device. The device must be writable.
    ///
    /// Dropping the IO device after calling this function will not close the standard output.
    pub fn from_io_ref<T: imp::AsRaw>(io: &T) -> io::Result<Self> {
        Self::from_raw(imp::as_raw(io))
    }
    /// Redirect standard output to the IO device. The device must be writable.
    pub fn from_io<T: imp::IntoRaw>(io: T) -> io::Result<Self> {
        Self::from_raw_owned(imp::into_raw(io))
    }
    /// Redirect the standard output to the file at that file path.
    ///
    /// The file will be created if it does not exist, and will be truncated if it does.
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::from_io(File::create(path)?)
    }
    /// Reset the standard output to its state before this type was constructed.
    ///
    /// This can be called to manually handle errors produced by the destructor.
    pub fn reset(mut self) -> io::Result<()> {
        self.reset_inner()?;
        self.reset = true;
        Ok(())
    }
    fn reset_inner(&self) -> io::Result<()> {
        imp::reset_stdout(imp::as_raw(&*self.original))
    }
}
impl Write for StdoutOverride {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.original.write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        self.original.write_vectored(bufs)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.original.flush()
    }
}
impl<'a> Write for &'a StdoutOverride {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&*self.original).write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        (&*self.original).write_vectored(bufs)
    }
    fn flush(&mut self) -> io::Result<()> {
        (&*self.original).flush()
    }
}
impl Drop for StdoutOverride {
    fn drop(&mut self) {
        if !self.reset {
            let _ = self.reset_inner();
        }
    }
}

/// An overridden standard error.
///
/// Writing to this writes to the original standard error. When it is dropped the standard error
/// will be reset.
#[derive(Debug)]
pub struct StderrOverride {
    original: ManuallyDrop<File>,
    reset: bool,
}
impl StderrOverride {
    /// Redirect standard error to the raw file descriptor. The file descriptor must be writable.
    ///
    /// The file descriptor is not owned, so it is your job to close it later. Closing it while
    /// this exists will not close the standard error.
    pub fn from_raw(raw: imp::Raw) -> io::Result<Self> {
        Ok(Self { original: ManuallyDrop::new(imp::override_stderr(raw, false)?), reset: false })
    }
    /// Redirect standard error to the owned raw file descriptor. The file descriptor must be
    /// writable.
    ///
    /// The file descriptor is owned, and so you must not use it after passing it to this function.
    pub fn from_raw_owned(raw: imp::Raw) -> io::Result<Self> {
        Ok(Self { original: ManuallyDrop::new(imp::override_stderr(raw, true)?), reset: false })
    }
    /// Redirect standard error to the IO device. The device must be writable.
    ///
    /// Dropping the IO device after calling this function will not close the standard error.
    pub fn from_io_ref<T: imp::AsRaw>(io: &T) -> io::Result<Self> {
        Self::from_raw(imp::as_raw(io))
    }
    /// Redirect standard error to the IO device. The device must be writable.
    pub fn from_io<T: imp::IntoRaw>(io: T) -> io::Result<Self> {
        Self::from_raw_owned(imp::into_raw(io))
    }
    /// Redirect the standard error to the file at that file path.
    ///
    /// The file will be created if it does not exist, and will be truncated if it does.
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::from_io(File::create(path)?)
    }
    /// Reset the standard error to its state before this type was constructed.
    ///
    /// This can be called to manually handle errors produced by the destructor.
    pub fn reset(mut self) -> io::Result<()> {
        self.reset_inner()?;
        self.reset = true;
        Ok(())
    }
    fn reset_inner(&self) -> io::Result<()> {
        imp::reset_stderr(imp::as_raw(&*self.original))
    }
}
impl Write for StderrOverride {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.original.write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        self.original.write_vectored(bufs)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.original.flush()
    }
}
impl<'a> Write for &'a StderrOverride {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&*self.original).write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        (&*self.original).write_vectored(bufs)
    }
    fn flush(&mut self) -> io::Result<()> {
        (&*self.original).flush()
    }
}
impl Drop for StderrOverride {
    fn drop(&mut self) {
        if !self.reset {
            let _ = self.reset_inner();
        }
    }
}

#[cfg(feature = "test-readme")]
doc_comment::doctest!("../README.md");

#[cfg(test)]
mod test {
    use crate::*;
    use std::io::{stderr, stdin, stdout, Read, Result, Write};

    use os_pipe::pipe;

    #[test]
    fn test_stdout() -> Result<()> {
        let (mut rx, tx) = pipe()?;
        let data = "12345";

        let guard = StdoutOverride::from_io_ref(&tx)?;
        print!("{}", data);
        stdout().flush()?;
        writeln!(&guard, "Outside! (1/2)")?;
        drop(guard);

        drop(tx);

        let mut contents = String::new();
        rx.read_to_string(&mut contents)?;
        assert_eq!(data, contents);
        println!("Outside! (2/2)");

        Ok(())
    }

    #[test]
    fn test_stderr() -> Result<()> {
        let (mut rx, tx) = pipe()?;
        let data = "123456";

        let guard = StderrOverride::from_io_ref(&tx)?;
        eprint!("{}", data);
        stderr().flush()?;
        writeln!(&guard, "Outside! (1/2)")?;
        drop(guard);

        drop(tx);

        let mut contents = String::new();
        rx.read_to_string(&mut contents)?;
        assert_eq!(data, contents);
        eprintln!("Outside! (2/2)");

        Ok(())
    }

    #[test]
    fn test_stdin() -> Result<()> {
        let (rx, mut tx) = pipe()?;
        let data = "12345\n";

        write!(&tx, "{}", data)?;
        tx.flush()?;

        let guard = StdinOverride::from_io(rx)?;

        print!("Please enter some text: ");
        stdout().flush()?;

        let mut s = String::new();
        stdin().read_line(&mut s)?;

        drop(guard);

        assert_eq!(data, s);

        println!("You typed: {}", s);

        Ok(())
    }
}
