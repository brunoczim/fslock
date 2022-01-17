#![cfg(feature = "std")]

use std::{ffi::c_void, fs::File, os::windows::io::FromRawHandle as _};

use crate::LockFile;

/// Turn the [`LockFile`] into a [`std::fs::File`]; you should probably also
/// call [`crate::lockfile_truncate`].
/// ```
#[doc = include_str!("../../examples/lock_preserved.rs")]
/// ```
impl Into<File> for LockFile {
    fn into(self) -> File {
        unsafe { File::from_raw_handle(self.raw() as *mut c_void) }
    }
}
