
use crate::Error;
use crate::sys::FileDesc;
use  crate::Exclusive;

#[derive(Debug,Copy,Clone)]
pub struct FileId;

impl FileId {
    pub(crate) fn get_id(_: FileDesc, _ : Exclusive) -> Result<Self, Error> {
        Ok(FileId)
    }
    pub fn take_lock(&self) {}
    pub fn try_take_lock(&self) -> bool { true }
    pub fn release_lock(&self) {}
}
