#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use crate::unix as sys;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use crate::windows as sys;

#[cfg(test)]
mod test;

pub use crate::sys::{Error, OsStr, OsString};
use core::{fmt, ops::Deref};

impl fmt::Debug for OsString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.as_ref())
    }
}

impl fmt::Display for OsString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_ref())
    }
}

impl Deref for OsString {
    type Target = OsStr;

    fn deref(&self) -> &OsStr {
        self.as_ref()
    }
}

/// Either borrowed or owned allocation of an OS-native string.
#[derive(Debug)]
pub enum EitherOsStr<'str> {
    /// Borrowed allocation.
    Borrowed(&'str OsStr),
    /// Owned allocation.
    Owned(OsString),
}

impl<'str> AsRef<OsStr> for EitherOsStr<'str> {
    fn as_ref(&self) -> &OsStr {
        match self {
            Self::Borrowed(str) => str,
            Self::Owned(string) => string.as_ref(),
        }
    }
}

impl<'str> Deref for EitherOsStr<'str> {
    type Target = OsStr;

    fn deref(&self) -> &OsStr {
        self.as_ref()
    }
}

/// Conversion of anything into an owned OS-native string. If allocation fails,
/// an error shall be returned.
pub trait IntoOsString {
    /// Converts with possible allocation error.
    fn into_os_string(self) -> Result<OsString, Error>;
}

impl IntoOsString for OsString {
    fn into_os_string(self) -> Result<OsString, Error> {
        Ok(self)
    }
}

impl<'str> IntoOsString for EitherOsStr<'str> {
    fn into_os_string(self) -> Result<OsString, Error> {
        match self {
            Self::Borrowed(str) => str.into_os_string(),
            Self::Owned(string) => Ok(string),
        }
    }
}

impl<'str> IntoOsString for &'str str {
    fn into_os_string(self) -> Result<OsString, Error> {
        self.to_os_str()?.into_os_string()
    }
}

/// Conversion of anything to an either borrowed or owned OS-native string. If
/// allocation fails, an error shall be returned.
pub trait ToOsStr {
    /// Converts with possible allocation error.
    fn to_os_str(&self) -> Result<EitherOsStr, Error>;
}

impl ToOsStr for OsStr {
    fn to_os_str(&self) -> Result<EitherOsStr, Error> {
        Ok(EitherOsStr::Borrowed(self))
    }
}

impl ToOsStr for OsString {
    fn to_os_str(&self) -> Result<EitherOsStr, Error> {
        Ok(EitherOsStr::Borrowed(self.as_ref()))
    }
}

#[derive(Debug)]
/// A handle to a file that is lockable. Does not delete the file.
/// # Example
/// ```
/// # fn main() -> Result<(), fslock::Error> {
/// use fslock::LockFile;
///
/// let mut file = LockFile::open("mylock")?;
/// file.lock()?;
/// do_stuff();
/// file.unlock()?;
///
/// # Ok(())
/// # }
/// # fn do_stuff() {
/// #    // doing stuff here.
/// # }
/// ```
pub struct LockFile {
    locked: bool,
    desc: sys::FileDesc,
}

impl LockFile {
    /// Opens a file for locking. On Unix, if the path is nul-terminated (ends
    /// with 0), no extra allocation will be made.
    ///
    /// # Panics
    /// Panics if the path contains a nul-byte in a place other than the end.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::LockFile;
    ///
    /// let mut file = LockFile::open("mylock")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::LockFile;
    ///
    /// let mut file = LockFile::open("my\0lock")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P>(path: &P) -> Result<Self, Error>
    where
        P: ToOsStr + ?Sized,
    {
        let path = path.to_os_str()?;
        let desc = sys::open(path.as_ref())?;
        Ok(Self { locked: false, desc })
    }

    /// Locks this file. Blocks while it is not possible to lock (i.e. someone
    /// else already owns a lock. After locked, if no attempt to unlock is made,
    /// it will be automatically unlocked on the file handle drop.
    ///
    /// # Panics
    /// Panics if this handle already owns the file.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::LockFile;
    ///
    /// let mut file = LockFile::open("mylock")?;
    /// file.lock()?;
    /// do_stuff();
    /// file.unlock()?;
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff() {
    /// #    // doing stuff here.
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::LockFile;
    ///
    /// let mut file = LockFile::open("mylock")?;
    /// file.lock()?;
    /// file.lock()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn lock(&mut self) -> Result<(), Error> {
        if self.locked {
            panic!("Cannot lock if already owning a lock");
        }
        sys::lock(self.desc)?;
        self.locked = true;
        Ok(())
    }

    /// Locks this file. Does NOT block if it is not possible to lock (i.e.
    /// someone else already owns a lock. After locked, if no attempt to
    /// unlock is made, it will be automatically unlocked on the file handle
    /// drop.
    ///
    /// # Panics
    /// Panics if this handle already owns the file.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::LockFile;
    ///
    /// let mut file = LockFile::open("mylock")?;
    /// if file.try_lock()? {
    ///     do_stuff();
    ///     file.unlock()?;
    /// }
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff() {
    /// #    // doing stuff here.
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::LockFile;
    ///
    /// let mut file = LockFile::open("mylock")?;
    /// file.lock()?;
    /// file.try_lock()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_lock(&mut self) -> Result<bool, Error> {
        if self.locked {
            panic!("Cannot lock if already owning a lock");
        }
        let locked = sys::try_lock(self.desc)?;
        if locked {
            self.locked = true;
        }
        Ok(locked)
    }

    /// Returns whether this file handle owns the lock.
    ///
    /// # Example
    /// ```
    /// use fslock::LockFile;
    /// # fn main() -> Result<(), fslock::Error> {
    ///
    /// let mut file = LockFile::open("mylock")?;
    /// do_stuff_with_lock(&mut file);
    /// if !file.owns_lock() {
    ///     file.lock()?;
    ///     do_stuff();
    ///     file.unlock()?;
    /// }
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff_with_lock(_lock: &mut LockFile) {
    /// #    // doing stuff here.
    /// # }
    /// # fn do_stuff() {
    /// #    // doing stuff here.
    /// # }
    /// ```
    pub fn owns_lock(&self) -> bool {
        self.locked
    }

    /// Unlocks this file. This file handle must own the file lock. If not
    /// called manually, it is automatically called on `drop`.
    ///
    /// # Panics
    /// Panics if this handle does not own the file.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::LockFile;
    ///
    /// let mut file = LockFile::open("mylock")?;
    /// file.lock()?;
    /// do_stuff();
    /// file.unlock()?;
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff() {
    /// #    // doing stuff here.
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::LockFile;
    ///
    /// let mut file = LockFile::open("mylock")?;
    /// file.unlock()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn unlock(&mut self) -> Result<(), Error> {
        if !self.locked {
            panic!("Attempted to unlock already locked lockfile");
        }
        sys::unlock(self.desc)?;
        self.locked = false;
        Ok(())
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        if self.locked {
            let _ = sys::unlock(self.desc);
        }
        sys::close(self.desc);
    }
}

#[derive(Debug)]
/// A handle to a file that is lockable. Deletes the file on drop. However,
/// while there are open handles to this file, it will not be deleted. The
/// deletion will happen after all handles/file descriptors are closed.
/// # Example
/// ```
/// # fn main() -> Result<(), fslock::Error> {
/// use fslock::TempLockFile;
///
/// let mut file = TempLockFile::open("mylock")?;
/// file.lock()?;
/// do_stuff();
/// file.unlock()?;
///
/// # Ok(())
/// # }
/// # fn do_stuff() {
/// #    // doing stuff here.
/// # }
/// ```
pub struct TempLockFile {
    inner: LockFile,
    path: OsString,
}

impl TempLockFile {
    /// Opens a file for locking. Removes the file on drop.
    ///
    /// # Panics
    /// Panics if the path contains a nul-byte in a place other than the end.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempLockFile;
    ///
    /// let mut file = TempLockFile::open("mylock")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempLockFile;
    ///
    /// let mut file = TempLockFile::open("my\0lock")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P>(path: P) -> Result<Self, Error>
    where
        P: IntoOsString,
    {
        let path = path.into_os_string()?;
        let inner = LockFile::open(&path)?;
        Ok(Self { inner, path })
    }

    /// Locks this file. Blocks while it is not possible to lock (i.e. someone
    /// else already owns a lock. After locked, if no attempt to unlock is made,
    /// it will be automatically unlocked on the file handle drop.
    ///
    /// # Panics
    /// Panics if this handle already owns the file.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempLockFile;
    ///
    /// let mut file = TempLockFile::open("mylock")?;
    /// file.lock()?;
    /// do_stuff();
    /// file.unlock()?;
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff() {
    /// #    // doing stuff here.
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempLockFile;
    ///
    /// let mut file = TempLockFile::open("mylock")?;
    /// file.lock()?;
    /// file.lock()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn lock(&mut self) -> Result<(), Error> {
        self.inner.lock()
    }

    /// Locks this file. Does NOT block if it is not possible to lock (i.e.
    /// someone else already owns a lock. After locked, if no attempt to
    /// unlock is made, it will be automatically unlocked on the file handle
    /// drop.
    ///
    /// # Panics
    /// Panics if this handle already owns the file.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempLockFile;
    ///
    /// let mut file = TempLockFile::open("mylock")?;
    /// if file.try_lock()? {
    ///     do_stuff();
    ///     file.unlock()?;
    /// }
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff() {
    /// #    // doing stuff here.
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempLockFile;
    ///
    /// let mut file = TempLockFile::open("mylock")?;
    /// file.lock()?;
    /// file.try_lock()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_lock(&mut self) -> Result<bool, Error> {
        self.inner.try_lock()
    }

    /// Returns whether this file handle owns the lock.
    ///
    /// # Example
    /// ```
    /// use fslock::TempLockFile;
    /// # fn main() -> Result<(), fslock::Error> {
    ///
    /// let mut file = TempLockFile::open("mylock")?;
    /// do_stuff_with_lock(&mut file);
    /// if !file.owns_lock() {
    ///     file.lock()?;
    ///     do_stuff();
    ///     file.unlock()?;
    /// }
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff_with_lock(_lock: &mut TempLockFile) {
    /// #    // doing stuff here.
    /// # }
    /// # fn do_stuff() {
    /// #    // doing stuff here.
    /// # }
    /// ```
    pub fn owns_lock(&self) -> bool {
        self.inner.owns_lock()
    }

    /// Unlocks this file. This file handle must own the file lock. If not
    /// called manually, it is automatically called on `drop`.
    ///
    /// # Panics
    /// Panics if this handle does not own the file.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempLockFile;
    ///
    /// let mut file = TempLockFile::open("mylock")?;
    /// file.lock()?;
    /// do_stuff();
    /// file.unlock()?;
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff() {
    /// #    // doing stuff here.
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempLockFile;
    ///
    /// let mut file = TempLockFile::open("mylock")?;
    /// file.unlock()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn unlock(&mut self) -> Result<(), Error> {
        self.inner.unlock()
    }
}

impl Drop for TempLockFile {
    fn drop(&mut self) {
        // We'll have to ignore this error.
        let _ = sys::remove(&self.path);
    }
}
