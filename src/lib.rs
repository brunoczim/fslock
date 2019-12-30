#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use crate::unix as sys;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use crate::windows as sys;

pub use crate::sys::{
    EitherOsStr,
    Error,
    IntoOsString,
    OsStr,
    OsString,
    ToOsStr,
};

#[derive(Debug)]
/// A handle to a file that is lockable. Does not delete the file.
/// # Example
/// ```
/// # fn main() -> Result<(), fslock::Error> {
/// use fslock::FileLock;
///
/// let mut file = FileLock::open("mylock")?;
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
pub struct FileLock {
    locked: bool,
    desc: sys::FileDesc,
}

impl FileLock {
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
    /// use fslock::FileLock;
    ///
    /// let mut file = FileLock::open("mylock")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::FileLock;
    ///
    /// let mut file = FileLock::open("my\0lock")?;
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
    /// use fslock::FileLock;
    ///
    /// let mut file = FileLock::open("mylock")?;
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
    /// use fslock::FileLock;
    ///
    /// let mut file = FileLock::open("mylock")?;
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
    /// use fslock::FileLock;
    ///
    /// let mut file = FileLock::open("mylock")?;
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
    /// use fslock::FileLock;
    ///
    /// let mut file = FileLock::open("mylock")?;
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
    /// use fslock::FileLock;
    /// # fn main() -> Result<(), fslock::Error> {
    ///
    /// let mut file = FileLock::open("mylock")?;
    /// do_stuff_with_lock(&mut file);
    /// if !file.owns_lock() {
    ///     file.lock()?;
    ///     do_stuff();
    ///     file.unlock()?;
    /// }
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff_with_lock(_lock: &mut FileLock) {
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
    /// use fslock::FileLock;
    ///
    /// let mut file = FileLock::open("mylock")?;
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
    /// use fslock::FileLock;
    ///
    /// let mut file = FileLock::open("mylock")?;
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

impl Drop for FileLock {
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
/// use fslock::TempFileLock;
///
/// let mut file = TempFileLock::open("mylock")?;
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
pub struct TempFileLock {
    inner: FileLock,
    path: OsString,
}

impl TempFileLock {
    /// Opens a file for locking. Removes the file on drop.
    ///
    /// # Panics
    /// Panics if the path contains a nul-byte in a place other than the end.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempFileLock;
    ///
    /// let mut file = TempFileLock::open("mylock")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::TempFileLock;
    ///
    /// let mut file = TempFileLock::open("my\0lock")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P>(path: P) -> Result<Self, Error>
    where
        P: IntoOsString,
    {
        let path = path.into_os_string()?;
        let inner = FileLock::open(&path)?;
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
    /// use fslock::TempFileLock;
    ///
    /// let mut file = TempFileLock::open("mylock")?;
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
    /// use fslock::TempFileLock;
    ///
    /// let mut file = TempFileLock::open("mylock")?;
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
    /// use fslock::TempFileLock;
    ///
    /// let mut file = TempFileLock::open("mylock")?;
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
    /// use fslock::TempFileLock;
    ///
    /// let mut file = TempFileLock::open("mylock")?;
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
    /// use fslock::TempFileLock;
    /// # fn main() -> Result<(), fslock::Error> {
    ///
    /// let mut file = TempFileLock::open("mylock")?;
    /// do_stuff_with_lock(&mut file);
    /// if !file.owns_lock() {
    ///     file.lock()?;
    ///     do_stuff();
    ///     file.unlock()?;
    /// }
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff_with_lock(_lock: &mut TempFileLock) {
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
    /// use fslock::TempFileLock;
    ///
    /// let mut file = TempFileLock::open("mylock")?;
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
    /// use fslock::TempFileLock;
    ///
    /// let mut file = TempFileLock::open("mylock")?;
    /// file.unlock()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn unlock(&mut self) -> Result<(), Error> {
        self.inner.unlock()
    }
}

impl Drop for TempFileLock {
    fn drop(&mut self) {
        // We'll have to ignore this error.
        let _ = sys::remove(&self.path);
    }
}
