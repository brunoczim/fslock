#[cfg(not(feature = "std"))]
use winapi::{
    shared::minwindef::{DWORD, TRUE},
    um::{
        synchapi::WaitForSingleObject,
        winbase::{
            FormatMessageW,
            FORMAT_MESSAGE_ALLOCATE_BUFFER,
            FORMAT_MESSAGE_FROM_SYSTEM,
            FORMAT_MESSAGE_IGNORE_INSERTS,
            WAIT_FAILED,
        },
        winnt::LANG_USER_DEFAULT,
        winnt::LPWSTR,
    },
};

use crate::{EitherOsStr, IntoOsString, ToOsStr};
use core::{
    convert::TryFrom,
    fmt::{self, Write},
    mem::MaybeUninit,
    ptr::{self, NonNull},
    slice,
};
use winapi::{
    shared::{minwindef::LPVOID, winerror::ERROR_LOCK_VIOLATION},
    um::{
        errhandlingapi::GetLastError,
        fileapi::{LockFileEx, UnlockFileEx},
        handleapi::CloseHandle,
        minwinbase::{
            LMEM_FIXED,
            LOCKFILE_EXCLUSIVE_LOCK,
            LOCKFILE_FAIL_IMMEDIATELY,
            LPOVERLAPPED,
            OVERLAPPED,
        },
        winbase::{LocalAlloc, LocalFree},
        winnt::{HANDLE, WCHAR},
    },
};

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
        Self::from_raw_os_error(unsafe { GetLastError() } as i32)
    }

    /// Raw OS error code. Returns option for compatibility with std.
    pub fn raw_os_error(&self) -> Option<i32> {
        Some(self.code)
    }
}

#[cfg(not(feature = "std"))]
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut buf: LPWSTR = ptr::null_mut();
        let res = unsafe {
            FormatMessageW(
                FORMAT_MESSAGE_ALLOCATE_BUFFER
                    | FORMAT_MESSAGE_FROM_SYSTEM
                    | FORMAT_MESSAGE_IGNORE_INSERTS,
                ptr::null_mut(),
                self.code as DWORD,
                LANG_USER_DEFAULT as DWORD,
                &mut buf as *mut LPWSTR as LPWSTR,
                0,
                ptr::null_mut(),
            )
        };

        if res == 0 {
            write!(fmt, "error getting error message")?;
        } else {
            {
                let slice = unsafe {
                    slice::from_raw_parts(buf as *const WCHAR, res as usize)
                };
                write_wide_str(fmt, slice)?;
            }
            unsafe {
                LocalFree(buf as LPVOID);
            }
        }

        Ok(())
    }
}

/// Owned allocation of an OS-native string.
pub struct OsString {
    alloc: NonNull<WCHAR>,
    /// Length without the nul-byte.
    len: usize,
}

impl Drop for OsString {
    fn drop(&mut self) {
        let ptr = self.alloc.as_ptr() as LPVOID;
        unsafe { LocalFree(ptr) }
    }
}

impl AsRef<OsStr> for OsString {
    fn as_ref(&self) -> &OsStr {
        unsafe {
            OsStr::from_slice(slice::from_raw_parts(
                self.alloc.as_ptr(),
                self.len,
            ))
        }
    }
}

/// Borrowed allocation of an OS-native string.
pub struct OsStr {
    chars: [WCHAR],
}

impl OsStr {
    /// Unsafe cause sequence needs to end with 0.
    unsafe fn from_slice(slice: &[WCHAR]) -> &Self {
        transmute(slice)
    }
}

impl fmt::Debug for OsStr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut first = false;
        let mut suplement = false;
        let mut prev = 0;
        write!(fmt, "[")?;

        for &char in &self.chars {
            if first {
                first = false;
            } else if !suplement {
                write!(fmt, ", ")?;
            }

            if suplement {
                let high = prev as u32 - 0xD800;
                let low = code as u32 - 0xDC00;
                let ch = char::try_from((high << 10 | low) + 0x10000)
                    .expect("Inconsistent char implementation");
                write!(fmt, "{:?}", ch)?;
            } else if code <= 0xD7FF || code >= 0xE000 {
                let ch = char::try_from(code as u32)
                    .expect("Inconsistent char implementation");
                write!(fmt, "{:?}", ch)?;
            } else {
                suplement = true;
                prev = code;
            }
        }

        write!(fmt, "]")?;
        Ok(())
    }
}

impl fmt::Display for OsStr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut suplement = false;
        let mut prev = 0;
        for &code in self.chars {
            if suplement {
                let high = prev as u32 - 0xD800;
                let low = code as u32 - 0xDC00;
                let ch = char::try_from((high << 10 | low) + 0x10000)
                    .expect("Inconsistent char implementation");
                write!(fmt, "{}", ch)?;
            } else if code <= 0xD7FF || code >= 0xE000 {
                let ch = char::try_from(code as u32)
                    .expect("Inconsistent char implementation");
                write!(fmt, "{}", ch)?;
            } else {
                suplement = true;
                prev = code;
            }
        }

        Ok(())
    }
}

impl<'str> IntoOsString for &'str OsStr {
    fn into_os_string(self) -> Result<OsString, Error> {
        let len = unsafe { libc::strlen(self.bytes.as_ptr()) };
        let alloc = unsafe { libc::malloc(len + 1) };
        let alloc = match NonNull::new(alloc as *mut WCHAR) {
            Some(alloc) => alloc,
            None => {
                return Err(Error::last_os_error());
            },
        };
        unsafe {
            libc::memcpy(
                alloc.as_ptr() as *mut libc::c_void,
                self.bytes.as_ptr() as *const libc::c_void,
                len * 2 + 2,
            );
        }

        Ok(OsString { alloc, len })
    }
}

impl ToOsStr for str {
    fn to_os_str(&self) -> Result<EitherOsStr, Error> {
        let len = self.encode_utf16().count();
        let alloc = unsafe { LocalAlloc(LMEM_FIXED, len * 2 + 2) };
        let alloc = match NonNull::new(alloc as *mut WCHAR) {
            Some(alloc) => alloc,
            None => {
                return Err(Error::last_os_error());
            },
        };

        let mut iter = self.encode_utf16();
        for i in 0 .. len {
            let ch = iter.next().expect("Inconsistent .encode_utf16()");
            unsafe {
                *alloc.as_ptr().add(i) = ch;
            }
        }
        unsafe {
            *alloc.as_ptr().add(len) = 0;
        }
        let string = OsString { alloc, len };
        Ok(EiteherOsStr::Owned(string))
    }
}

/// Opens a file with only purpose of locking it. Creates it if it does not
/// exist. Path must not contain a nul-byte in the middle, but a nul-byte in the
/// end (and only in the end) is allowed, which in this case no extra allocation
/// will be made. Otherwise, an extra allocation is made.
pub fn open(path: &OsStr) -> Result<FileDesc, Error> {
    unimplemented!()
}

/// Tries to lock a file and blocks until it is possible to lock.
pub fn lock(handle: FileDesc) -> Result<(), Error> {
    let mut overlapped: OVERLAPPED =
        unsafe { MaybeUninit::zeroed().assume_init() };
    unsafe {
        overlapped.u.s_mut().Offset = 0;
        overlapped.u.s_mut().OffsetHigh = 0;
    }
    overlapped.hEvent = ptr::null_mut();

    let res = unsafe {
        LockFileEx(
            handle,
            LOCKFILE_EXCLUSIVE_LOCK,
            0,
            DWORD::max_value(),
            DWORD::max_value(),
            &mut overlapped as LPOVERLAPPED,
        )
    };
    if res == TRUE {
        let res = unsafe { WaitForSingleObject(overlapped.hEvent, 0) };
        if res != WAIT_FAILED {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    } else {
        Err(Error::last_os_error())
    }
}

/// Tries to lock a file but returns as soon as possible if already locked.
pub fn try_lock(handle: FileDesc) -> Result<bool, Error> {
    let mut overlapped: OVERLAPPED =
        unsafe { MaybeUninit::zeroed().assume_init() };
    unsafe {
        overlapped.u.s_mut().Offset = 0;
        overlapped.u.s_mut().OffsetHigh = 0;
    }
    overlapped.hEvent = ptr::null_mut();

    let res = unsafe {
        LockFileEx(
            handle,
            LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0,
            DWORD::max_value(),
            DWORD::max_value(),
            &mut overlapped as LPOVERLAPPED,
        )
    };
    if res == TRUE {
        let res = unsafe { WaitForSingleObject(overlapped.hEvent, 0) };
        if res != WAIT_FAILED {
            Ok(true)
        } else {
            Err(Error::last_os_error())
        }
    } else {
        let err = unsafe { GetLastError() };
        if err == ERROR_LOCK_VIOLATION {
            Ok(false)
        } else {
            Err(Error::from_raw_os_error(err as i32))
        }
    }
}

/// Unlocks the file.
pub fn unlock(handle: FileDesc) -> Result<(), Error> {
    let mut overlapped: OVERLAPPED =
        unsafe { MaybeUninit::zeroed().assume_init() };
    unsafe {
        overlapped.u.s_mut().Offset = 0;
        overlapped.u.s_mut().OffsetHigh = 0;
    }
    overlapped.hEvent = ptr::null_mut();

    let res = unsafe {
        UnlockFileEx(
            handle,
            0,
            DWORD::max_value(),
            DWORD::max_value(),
            &mut overlapped as LPOVERLAPPED,
        )
    };
    if res == TRUE {
        let res = unsafe { WaitForSingleObject(overlapped.hEvent, 0) };
        if res != WAIT_FAILED {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    } else {
        Err(Error::last_os_error())
    }
}

/// Removes a file. Path must not contain a nul-byte in the middle, but a
/// nul-byte in the end (and only in the end) is allowed, which in this case no
/// extra allocation will be made. Otherwise, an extra allocation is made.
pub fn remove(path: &OsStr) -> Result<(), Error> {
    unimplemented!()
}

/// Closes the file.
pub fn close(handle: FileDesc) {
    unsafe {
        CloseHandle(handle);
    }
}
