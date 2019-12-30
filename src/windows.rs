use winapi::um::{errhandlingapi::GetLastError, winnt::HANDLE};

/// A type representing file descriptor on Unix.
pub type FileDesc = HANDLE;

#[cfg(feature = "std")]
/// An IO error.
pub type Error = std::io::Error;

#[cfg(not(feature = "std"))]
#[derive(Debug)]
/// An IO error. Without std, you can only get a message or an OS error code.
pub struct Error {
    code: i32,
}

#[cfg(not(feature = "std"))]
impl Error {
    /// Creates an error from a raw OS error code.
    pub fn from_raw_os_error(code: i32) -> Self {
        Self { code }
    }

    /// Creates an error from the last OS error code.
    pub fn last_os_error() -> Error {
        Self::from_raw_os_error(unsafe { GetLastError() })
    }

    /// Raw OS error code. Returns option for compatibility with std.
    pub fn raw_os_error(&self) -> Option<i32> {
        Some(self.code)
    }
}

#[cfg(not(feature = "std"))]
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

/// Opens a file with only purpose of locking it. Creates it if it does not
/// exist. Path must not contain a nul-byte in the middle, but a nul-byte in the
/// end (and only in the end) is allowed, which in this case no extra allocation
/// will be made. Otherwise, an extra allocation is made.
pub fn open(path: &[u8]) -> Result<FileDesc, Error> {
    unimplemented!()
}

/// Tries to lock a file and blocks until it is possible to lock.
pub fn lock(fd: FileDesc) -> Result<(), Error> {
    unimplemented!()
}

/// Tries to lock a file but returns as soon as possible if already locked.
pub fn try_lock(fd: FileDesc) -> Result<bool, Error> {
    unimplemented!()
}

/// Unlocks the file.
pub fn unlock(fd: FileDesc) -> Result<(), Error> {
    unimplemented!()
}

/// Removes a file. Path must not contain a nul-byte in the middle, but a
/// nul-byte in the end (and only in the end) is allowed, which in this case no
/// extra allocation will be made. Otherwise, an extra allocation is made.
pub fn remove(path: &[u8]) -> Result<(), Error> {
    unimplemented!()
}

/// Closes the file.
pub fn close(fd: FileDesc) {
    unimplemented!()
}
