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
//!     use stdio_override::StdoutOverride;
//!     use std::{fs, mem};
//!     let file_name = "./test.txt";
//!     let guard = StdoutOverride::override_file(file_name).unwrap();
//!      println!("Isan to Stdout!");
//!
//!     let contents = fs::read_to_string(file_name).unwrap();
//!     assert_eq!("Isan to Stdout!\n", contents);
//!     mem::drop(guard);
//!     println!("Outside!");

//! ```
//!
mod ffi;
#[macro_use]
mod macros;

fd_guard!(StdoutOverride, guard: StdoutOverrideGuard, FD: crate::ffi::STDOUT_FILENO, name of FD: stdout);
fd_guard!(StdinOverride, guard: StdinOverrideGuard, FD: crate::ffi::STDIN_FILENO, name of FD: stdin);
fd_guard!(StderrOverride, guard: StderrOverrideGuard, FD: crate::ffi::STDERR_FILENO, name of FD: stderr);

pub use crate::stderr::{StderrOverride, StderrOverrideGuard};


pub use crate::stdout::{StdoutOverride, StdoutOverrideGuard};

#[cfg(feature = "test-readme")]
doc_comment::doctest!("../README.md");
