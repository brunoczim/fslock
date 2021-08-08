
use crate::Error;
use crate::sys::FileDesc;

use std::{sync::{Arc, Mutex, Condvar}, collections::{HashMap, hash_map::Entry}, mem::MaybeUninit};
use once_cell::sync::Lazy;

pub type FileId = (libc::dev_t, libc::ino_t);

static HELD_LOCKS: Lazy<Mutex<HashMap<FileId, Arc<Condvar>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn get_id(fd: FileDesc) -> Result<FileId, Error> {
    let mut stat = MaybeUninit::<libc::stat>::zeroed();
    let result_code = unsafe { libc::fstat(fd, stat.as_mut_ptr()) };
    if result_code >= 0 {
        let stat = unsafe { stat.assume_init() };
        Ok((stat.st_dev, stat.st_ino))
    } else {
        Err(Error::last_os_error())
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

