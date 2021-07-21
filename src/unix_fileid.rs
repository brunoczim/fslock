
use crate::Error;
use crate::sys::FileDesc;

use std::{sync::{Arc, Mutex, Condvar}, collections::{HashMap, hash_map::Entry}};
use once_cell::sync::Lazy;

pub type FileId = (u64, u64);

static HELD_LOCKS: Lazy<Mutex<HashMap<FileId, Arc<Condvar>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn get_id(fd: FileDesc) -> Result<FileId, Error> {
    unsafe {
        let mut stat: libc::stat = std::mem::zeroed();
        if libc::fstat(fd, &mut stat) >= 0 {
            Ok((stat.st_dev as u64, stat.st_ino as u64))
        } else {
            Err(Error::last_os_error())
        }
    }
}

pub fn take_lock(id: FileId) {
    let mut cvar: Option<Arc<Condvar>> = None;
    let mut held = HELD_LOCKS.lock().unwrap();
    loop {
        match held.entry(id) {
            Entry::Vacant(e) => {
                e.insert(cvar.unwrap_or_else(|| Arc::new(Condvar::new())));
                return;
            }
            Entry::Occupied(ref e) => {
                let cv = Arc::clone(e.get());
                held = cv.wait(held).unwrap(); // releases lock on held while waiting.
                cvar = Some(cv);
            }
        }
    }
}

pub fn try_take_lock(id: FileId) -> bool{
    let mut held = HELD_LOCKS.lock().unwrap();
    if let Entry::Vacant(e) = held.entry(id) {
        e.insert(Arc::new(Condvar::new()));
        true
    } else {
        false
    }
}

pub fn release_lock(id: FileId) {
    let mut held = HELD_LOCKS.lock().unwrap();
    if let Some(cvar) = held.remove(&id) {
        cvar.notify_one();
    }
}

