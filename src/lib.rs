#[cfg(unix)]
mod unix;
#[cfg(unix)]
use crate::unix as sys;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use crate::windows as sys;

pub use crate::sys::Error;

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
    /// Opens a file for locking. If the path is not nul-terminated (ends with
    /// 0), an extra allocation will be made.
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
    /// # Example Without Extra Allocation
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::FileLock;
    ///
    /// let mut file = FileLock::open("mylock\0")?;
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
        P: AsRef<[u8]> + ?Sized,
    {
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

#[cfg(feature = "std")]
#[derive(Debug)]
/// A handle to a file that is lockable. Deletes the file on drop.
/// # Example
/// ```
/// # fn main() -> Result<(), fslock::Error> {
/// use fslock::SelfDestroyFileLock;
///
/// let mut file = SelfDestroyFileLock::open("mylock")?;
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
pub struct SelfDestroyFileLock {
    inner: FileLock,
    name: Vec<u8>,
}

#[cfg(feature = "std")]
impl SelfDestroyFileLock {
    /// Opens a file for locking. Even if the path contains a trailing nul-byte
    /// (0), an extra allocation will be made to store the path, so that one can
    /// remove it on drop.
    ///
    /// # Panics
    /// Panics if the path contains a nul-byte in a place other than the end.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::SelfDestroyFileLock;
    ///
    /// let mut file = SelfDestroyFileLock::open("mylock")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example Without Extra Allocation
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::SelfDestroyFileLock;
    ///
    /// let mut file = SelfDestroyFileLock::open("mylock\0")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panicking Example
    ///
    /// ```should_panic
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::SelfDestroyFileLock;
    ///
    /// let mut file = SelfDestroyFileLock::open("my\0lock")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P>(path: P) -> Result<Self, Error>
    where
        P: Into<Vec<u8>>,
    {
        let mut path = path.into();
        if path.last() != Some(&0) {
            path.push(0);
        }
        let inner = FileLock::open(&path)?;
        Ok(Self { inner, name: path })
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
    /// use fslock::SelfDestroyFileLock;
    ///
    /// let mut file = SelfDestroyFileLock::open("mylock")?;
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
    /// use fslock::SelfDestroyFileLock;
    ///
    /// let mut file = SelfDestroyFileLock::open("mylock")?;
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
    /// use fslock::SelfDestroyFileLock;
    ///
    /// let mut file = SelfDestroyFileLock::open("mylock")?;
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
    /// use fslock::SelfDestroyFileLock;
    ///
    /// let mut file = SelfDestroyFileLock::open("mylock")?;
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
    /// use fslock::SelfDestroyFileLock;
    /// # fn main() -> Result<(), fslock::Error> {
    ///
    /// let mut file = SelfDestroyFileLock::open("mylock")?;
    /// do_stuff_with_lock(&mut file);
    /// if !file.owns_lock() {
    ///     file.lock()?;
    ///     do_stuff();
    ///     file.unlock()?;
    /// }
    ///
    /// # Ok(())
    /// # }
    /// # fn do_stuff_with_lock(_lock: &mut SelfDestroyFileLock) {
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
    /// use fslock::SelfDestroyFileLock;
    ///
    /// let mut file = SelfDestroyFileLock::open("mylock")?;
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
    /// use fslock::SelfDestroyFileLock;
    ///
    /// let mut file = SelfDestroyFileLock::open("mylock")?;
    /// file.unlock()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn unlock(&mut self) -> Result<(), Error> {
        self.inner.unlock()
    }
}

#[cfg(feature = "std")]
impl Drop for SelfDestroyFileLock {
    fn drop(&mut self) {
        let _ = sys::remove(&self.name);
    }
}
