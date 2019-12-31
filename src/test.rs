extern crate std;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering::*},
        Arc,
    },
    thread,
    time::Duration,
};
use crate::{LockFile, TempLockFile};

#[test]
fn lock() {
    let shared = Arc::new(AtomicBool::new(false));

    let mut file = LockFile::open("lock.test").unwrap();
    file.lock().unwrap();

    {
        let shared = shared.clone();
        thread::spawn(move || {
            let mut file = LockFile::open("lock.test").unwrap();
            file.lock().unwrap();
            shared.store(true, SeqCst);
            file.unlock().unwrap();
        });
    }

    thread::sleep(Duration::from_millis(50));
    assert!(!shared.load(SeqCst));
    file.unlock().unwrap();

    thread::sleep(Duration::from_millis(50));
    file.lock().unwrap();

    assert!(shared.load(SeqCst));
}
