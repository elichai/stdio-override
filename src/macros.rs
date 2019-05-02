macro_rules! fd_guard {
    ($name:ident, guard: $guard_name:ident, FD: $fd:path, name of FD: $doc:ident) => {
        mod $doc {
            use crate::ffi::{close, dup, dup2};
            use std::fs::OpenOptions;
            use std::io;
            use std::os::unix::io::IntoRawFd;
            use std::os::unix::io::RawFd;
            use std::os::unix::prelude::*;
            use std::path::Path;
            use std::sync::atomic::{AtomicBool, Ordering};

            static IS_REPLACED: AtomicBool = AtomicBool::new(false);

            const ORDERING: Ordering = Ordering::SeqCst;

            /// A Guard over the File Descriptor change.
            /// when this guard is dropped the File Descriptor will go back to the original,
            /// and the file will be closed.
            ///
            /// For more information please see the [module-level documentation]
            ///
            /// [module-level documentation]: index.html
            pub struct $guard_name {
                original_fd: RawFd,
                file_fd: RawFd,
            }

            /// Override the File Descriptor safely.
            /// For more information please see the [module-level documentation]
            ///
            /// [module-level documentation]: index.html
            pub struct $name;

            impl $name {
                /// Override the File Descriptor by providing a path.
                /// This uses [`OpenOptions`] with `create(true)` so it will fail/succeed accordingly.
                /// (won't fail if the fail already exists and will create it if it doesn't exist.)
                ///
                /// [`File::create`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html
                pub fn override_file<P: AsRef<Path>>(p: P) -> io::Result<$guard_name> {
                    Self::check_and_override();

                    let file = OpenOptions::new().read(true).write(true).append(true).create(true).open(p)?;
                    let file_fd = file.into_raw_fd();
                    Self::override_fd(file_fd)
                }

                /// Override the File Descriptor by providing something that can be turned into a file descriptor.
                /// This will accept Sockets, Files, and even Stdio's. [`AsRawFd`]
                ///
                /// [`AsRawFd`]: https://doc.rust-lang.org/stable/std/os/unix/io/trait.AsRawFd.html
                pub fn override_raw<FD: AsRawFd>(fd: FD) -> io::Result<$guard_name> {
                    Self::check_and_override();

                    let file_fd = fd.as_raw_fd();
                    Self::override_fd(file_fd)
                }

                fn override_fd(file_fd: RawFd) -> io::Result<$guard_name> {
                    let original_fd = unsafe { dup($fd) }?;
                    let _ = unsafe { dup2(file_fd, $fd) }?;

                    IS_REPLACED.store(true, ORDERING);

                    Ok($guard_name { original_fd, file_fd })
                }

                fn check_and_override() {
                    match IS_REPLACED.compare_exchange(false, true, ORDERING, ORDERING) {
                        Ok(_) => (),
                        Err(_e) => panic!("Tried to override Stdout twice"),
                    }
                }
            }

            impl Drop for $guard_name {
                fn drop(&mut self) {
                    // Ignoring syscalls errors seems to be the most sensible thing to do in a Drop impl
                    // https://github.com/rust-lang/rust/blob/bd177f3e/src/libstd/sys/unix/fd.rs#L293-L302
                    let _ = unsafe { dup2(self.original_fd, $fd) };
                    let _ = unsafe { close(self.file_fd) };
                    IS_REPLACED.store(false, ORDERING);
                }
            }

        }
    };
}
