use core::cell::RefCell;

// A constant controlling whether the ``LockFile::open`` function truncates
// files when it is closed/unlocked. On `no_std`, mutation is unsafe.
#[cfg(feature = "std")]
std::thread_local!(
pub static DEFAULT_LOCKFILE_TRUNCATE: RefCell<bool> = RefCell::new(true)
);
#[cfg(not(feature = "std"))]
pub static mut DEFAULT_LOCKFILE_TRUNCATE: RefCell<bool> = RefCell::new(true);

#[cfg(feature = "std")]
pub(crate) fn default_lockfile_truncate_state() -> bool {
    DEFAULT_LOCKFILE_TRUNCATE.with(|b| *b.borrow())
}
#[cfg(not(feature = "std"))]
pub(crate) unsafe fn default_lockfile_truncate_state() -> bool {
    *DEFAULT_LOCKFILE_TRUNCATE.borrow()
}

/// Change the state of default file truncation (default true). On `#[no_std]`,
/// mutation is unsafe!
#[cfg(feature = "std")]
pub fn lockfile_truncate(dlft: bool) {
    DEFAULT_LOCKFILE_TRUNCATE.with(|b| *b.borrow_mut() = dlft)
}
#[cfg(not(feature = "std"))]
pub unsafe fn lockfile_truncate(dlft: bool) {
    *DEFAULT_LOCKFILE_TRUNCATE.borrow_mut() = dlft;
}
