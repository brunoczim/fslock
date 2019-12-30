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
pub struct LockFile {
    desc: sys::FileDesc,
}

impl LockFile {
    pub fn new<P>(path: &P) -> Result<Self, Error>
    where
        P: AsRef<[u8]> + ?Sized,
    {
        let desc = sys::open(path.as_ref())?;
        Ok(Self { desc })
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        sys::close(self.desc);
    }
}
