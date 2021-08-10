use crate::{sys::FileDesc, Error, Exclusivity};

use once_cell::sync::Lazy;
use std::{
    collections::{hash_map::Entry, HashMap},
    mem::MaybeUninit,
    sync::{Arc, Condvar, Mutex},
};

type RawFileId = (libc::dev_t, libc::ino_t);

static HELD_LOCKS: Lazy<Mutex<HashMap<RawFileId, Arc<Condvar>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn get_raw_id(fd: FileDesc) -> Result<RawFileId, Error> {
    let mut stat = MaybeUninit::<libc::stat>::zeroed();
    let result_code = unsafe { libc::fstat(fd, stat.as_mut_ptr()) };
    if result_code >= 0 {
        let stat = unsafe { stat.assume_init() };
        Ok((stat.st_dev, stat.st_ino))
    } else {
        Err(Error::last_os_error())
    }
}

fn take_lock(id: RawFileId) {
    let mut cvar: Option<Arc<Condvar>> = None;
    let mut held = HELD_LOCKS.lock().unwrap();
    loop {
        match held.entry(id) {
            Entry::Vacant(e) => {
                e.insert(cvar.unwrap_or_else(|| Arc::new(Condvar::new())));
                return;
            },
            Entry::Occupied(ref e) => {
                let cv = Arc::clone(e.get());
                held = cv.wait(held).unwrap(); // releases lock on held while waiting.
                cvar = Some(cv);
            },
        }
    }
}

fn try_take_lock(id: RawFileId) -> bool {
    let mut held = HELD_LOCKS.lock().unwrap();
    if let Entry::Vacant(e) = held.entry(id) {
        e.insert(Arc::new(Condvar::new()));
        true
    } else {
        false
    }
}

fn release_lock(id: RawFileId) {
    let mut held = HELD_LOCKS.lock().unwrap();
    if let Some(cvar) = held.remove(&id) {
        cvar.notify_one();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FileId {
    Exclusive(RawFileId),
    NonExclusive,
}

impl FileId {
    pub(crate) fn get_id(fd: FileDesc, ex: Exclusivity) -> Result<Self, Error> {
        match ex {
            Exclusivity::PerFileDesc => Ok(FileId::Exclusive(get_raw_id(fd)?)),
            Exclusivity::OsDependent => Ok(FileId::NonExclusive),
        }
    }
    pub fn take_lock(&self) {
        match self {
            FileId::NonExclusive => {},
            FileId::Exclusive(raw) => take_lock(*raw),
        }
    }
    pub fn try_take_lock(&self) -> bool {
        match self {
            FileId::NonExclusive => true,
            FileId::Exclusive(raw) => try_take_lock(*raw),
        }
    }
    pub fn release_lock(&self) {
        match self {
            FileId::NonExclusive => {},
            FileId::Exclusive(raw) => release_lock(*raw),
        }
    }
}
