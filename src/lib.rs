#![cfg_attr(not(feature = "std"), no_std)]

//! API to use files as a lock. Supports non-std crates by disabling feature
//! `std`.
//!
//! # Types
//! Currently, only one type is provided: [`LockFile`]. It does not destroy the
//! file after closed and behaviour on locking different file handles owned by
//! the same process is different between Unix and Windows. # Example:
//!
//! # Example
//! ```
//! use fslock::LockFile;
//! fn main() -> Result<(), fslock::Error> {
//!
//!     let mut file = LockFile::open("mylock.test")?;
//!     file.lock()?;
//!     do_stuff();
//!     file.unlock()?;
//!
//!     Ok(())
//! }
//! # fn do_stuff() {
//! #    // doing stuff here.
//! # }
//! ```

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use crate::unix as sys;

#[cfg(all(unix, feature = "multilock"))]
mod unix_fileid;
#[cfg(all(unix, feature = "multilock"))]
use unix_fileid as fileid;
#[cfg(not(all(unix, feature = "multilock")))]
mod nil_fileid;
#[cfg(not(all(unix, feature = "multilock")))]
use nil_fileid as fileid;

mod string;
mod fmt;

/// Enumeration used to declare whether FsLock instances opened with the same
/// file, by the same process, are exclusive.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
enum Exclusivity {
    /// Treat any two file descriptors to the same file as having
    /// separate locks.
    ///
    /// This option requires allocation internally, and is not
    /// available on Unix when building without the `std` feature.
    #[cfg(any(not(unix), feature = "multilock"))]
    PerFileDesc,
    /// Os-dependent behavior.
    OsDependent,
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use crate::windows as sys;

pub use crate::{
    string::{EitherOsStr, IntoOsString, ToOsStr},
    sys::{Error, OsStr, OsString},
};

#[derive(Debug)]
/// A handle to a file that is lockable. Does not delete the file.
///
/// # Multiple Handles/Descriptors To The Same File
///
/// The underlying file locking code behaves differently on Windows
/// and Unix when the same process tries to lock the same file via two
/// different LockFiles.  See [`LockFile::open()`] for more
/// information.  You can work around this OS dependency by using
/// [`LockFile::open_excl()`].
///
/// # Example
/// ```
/// # fn main() -> Result<(), fslock::Error> {
/// use fslock::LockFile;
///
/// let mut file = LockFile::open("mylock.test")?;
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
    id: fileid::FileId,
    desc: sys::FileDesc,
}

impl LockFile {
    /// Opens a file for locking. Exclusive here means the lock is exclusive to
    /// a file descriptor/handle on all platforms, instead of Unix's behaviour
    /// of locking for the whole process. Do not confuse "exclusive" with the
    /// terminology of Linux's and BSD's flock system call.
    ///
    /// On Unix, if the path is nul-terminated (ends with 0), no extra
    /// allocation will be made.
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
    /// let mut file = LockFile::open_excl("mylock.test")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example with multiple locks.
    ///
    /// ```
    /// # fn main() -> Result<(), fslock::Error> {
    /// use fslock::LockFile;
    ///
    /// let mut lock1 = LockFile::open_excl("mylock.test")?;
    /// let mut lock2 = LockFile::open_excl("mylock.test")?;
    ///
    /// lock1.lock()?;
    /// // We're holding the lock via lock1: locking via lock2 will fail.
    /// assert_eq!(lock2.try_lock()?, false);
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
    ///
    /// # Availability
    ///
    /// This function is only available on Unix when the `multilock`
    /// feature is enabled.
    #[cfg(any(not(unix), feature = "multilock"))]
    pub fn open_excl<P>(path: &P) -> Result<Self, Error>
    where
        P: ToOsStr + ?Sized,
    {
        Self::open_internal(path, Exclusivity::PerFileDesc)
    }

    /// Opens a file for locking, with OS-dependent locking behavior. On Unix,
    /// if the path is nul-terminated (ends with 0), no extra
    /// allocation will be made.
    ///
    /// # Multiple Handles/Descriptors to the same file.
    ///
    /// This function replicates the underlying OS behavior from file
    /// locking, which gives different results on Windows and Unix
    /// when the same process tries to lock the same file more than
    /// once.
    ///
    /// Windows treats each _handle_ to a file as having its own lock,
    /// whereas Unix treats all descriptors for a file as sharing a
    /// lock for the whole process.  This means that on Windows you may
    /// open a file, lock it, open it again, and when you try to lock the
    /// second handle, it will block until the first lock is
    /// released. Meanwhile, Unix will check whether your process already
    /// owns the look, see that you already locked the file, and simply
    /// return as you already have the lock! It will only block if there
    /// is a _different_ process holding the lock. Also, unlocking one
    /// file descriptor on unix will unlock the file for the whole
    /// process.
    ///
    /// For consistent behavior across operating systems, you can
    /// either make sure that the same file is never locked more than
    /// once by the same process, or you can use the
    /// [`LockFile::open_excl()`] call instead (which requires
    /// `multilock` and `std` on Unix).
    ///
    /// # Compatibility
    ///
    /// The lock files returned by this method can exhibit
    /// OS-dependent behavior: See "Multiple Handles/Descriptors To
    /// The Same File" in the documentation for [`LockFile`].
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
    /// let mut file = LockFile::open("mylock.test")?;
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
        Self::open_internal(path, Exclusivity::OsDependent)
    }

    /// Implementation helper for open_excl and open.
    fn open_internal<P>(path: &P, ex: Exclusivity) -> Result<Self, Error>
    where
        P: ToOsStr + ?Sized,
    {
        let path = path.to_os_str()?;
        let desc = sys::open(path.as_ref())?;
        let id = fileid::FileId::get_id(desc, ex)?;
        Ok(Self { locked: false, id, desc })
    }

    /// Locks this file. Blocks while it is not possible to lock (i.e. someone
    /// else already owns a lock). After locked, if no attempt to unlock is
    /// made, it will be automatically unlocked on the file handle drop.
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
    /// let mut file = LockFile::open("mylock.test")?;
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
    /// let mut file = LockFile::open("mylock.test")?;
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
        self.id.take_lock();
        // We got the fileid lock; now try to lock the file.
        if let Err(error) = sys::lock(self.desc) {
            self.id.release_lock();
            return Err(error);
        }
        self.locked = true;
        Ok(())
    }

    /// Locks this file and writes this process's PID into the file, which will
    /// be erased on unlock. Like [`LockFile::lock`], blocks while it is not
    /// possible to lock. After locked, if no attempt to unlock is made, it will
    /// be automatically unlocked on the file handle drop.
    pub fn lock_with_pid(&mut self) -> Result<(), Error> {
        if let Err(error) = self.lock() {
            return Err(error);
        }

        let result = write!(fmt::Writer(self.desc), "{}", sys::pid());
        if result.is_err() {
            let _ = self.unlock();
        }
        result
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
    /// let mut file = LockFile::open("mylock.test")?;
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
    /// let mut file = LockFile::open("mylock.test")?;
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
        if self.id.try_take_lock() {
            // We got the fileid lock; now try to lock the file.
            let lock_result = sys::try_lock(self.desc);
            match lock_result {
                Ok(true) => self.locked = true,
                _ => self.id.release_lock(),
            }
            lock_result
        } else {
            Ok(false)
        }
    }

    /// Returns whether this file handle owns the lock.
    ///
    /// # Example
    /// ```
    /// use fslock::LockFile;
    /// # fn main() -> Result<(), fslock::Error> {
    ///
    /// let mut file = LockFile::open("mylock.test")?;
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
    /// let mut file = LockFile::open("mylock.test")?;
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
    /// let mut file = LockFile::open("mylock.test")?;
    /// file.unlock()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn unlock(&mut self) -> Result<(), Error> {
        if !self.locked {
            panic!("Attempted to unlock already locked lockfile");
        }
        self.locked = false;
        sys::unlock(self.desc)?;
        sys::truncate(self.desc)?;
        self.id.release_lock();
        Ok(())
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        if self.locked {
            let _ = sys::unlock(self.desc);
            self.id.release_lock();
            self.locked = false;
        }
        sys::close(self.desc);
    }
}

// Safe because:
// 1. We never actually access the contents of the pointer that represents the
// Windows Handle.
//
// 2. We require a mutable reference to actually mutate the file
// system.

#[cfg(windows)]
unsafe impl Send for LockFile {}

#[cfg(windows)]
unsafe impl Sync for LockFile {}

#[cfg(test)]
mod test {

    #[cfg(all(feature = "std", any(not(unix), feature = "multilock")))]
    #[test]
    fn exclusive_lock_cases() -> Result<(), crate::Error> {
        let mut f1 = crate::LockFile::open_excl("lock2.test")?;
        let mut f2 = crate::LockFile::open_excl("lock2.test")?;

        // f1 will get the lock; f2 can't.
        assert!(f1.try_lock()?);
        assert!(!f2.try_lock()?);

        // have f2 wait for f1.
        let thr = std::thread::spawn(move || {
            f2.lock().unwrap();
            f2
        });

        // Sleep here a little, so that the other thread has time to
        // block on the fd-lock.
        std::thread::sleep(std::time::Duration::from_millis(100));
        drop(f1); // Causes f1 to unlock.

        let f2 = thr.join().unwrap();

        assert!(f2.owns_lock());

        Ok(())
    }
}
