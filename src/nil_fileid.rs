
use crate::Error;
use crate::sys::FileDesc;

pub type FileId = ();
pub fn get_id(_: FileDesc) -> Result<FileId, Error> { Ok(()) }
pub fn take_lock(_: FileId) {}
pub fn try_take_lock(_: FileId) -> bool { true }
pub fn release_lock(_: FileId) {}
