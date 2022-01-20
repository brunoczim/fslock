#![cfg(feature = "std")]

use std::fs::File;
use std::os::unix::io::FromRawFd as _;
use crate::LockFile;

/// Turn the [`LockFile`] into a [`std::fs::File`]; you should probably also call
/// [`crate::lockfile_truncate`].
/// ```
#[doc = include_str!("../../examples/lock_preserved.rs")]
/// ```
impl Into<File> for &mut LockFile {
    fn into(self) -> File {
        unsafe {
            File::from_raw_fd(self.raw())
        }
    }
}
