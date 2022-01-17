use fslock::LockFile;
use std::{
    fs::File,
    io::{Read, Write},
};

fn main() -> Result<(), fslock::Error> {
    fslock::lockfile_truncate(false); // turn off truncation
    {
        let mut lock = LockFile::open("testfiles/preserved.lock")?;
        lock.lock()?;
        unsafe {
            assert!(lock.raw() != -1);
        }
        let mut file: File = (&mut lock).into();
        file.write(b"the \xF0\x9F\x90\xAE says moo")?;
        file.sync_all()?;
    } // drop the lock and the writable file
      // open a readable file
    let mut s = String::new();
    File::open("testfiles/preserved.lock").unwrap().read_to_string(&mut s)?;
    assert_eq!(s, "the 🐮 says moo");
    Ok(())
}
