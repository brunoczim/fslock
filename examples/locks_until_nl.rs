use fslock::LockFile;
use std::io::{self, Read};

fn main() -> Result<(), fslock::Error> {
    let mut lockfile = LockFile::open("examplelock.test")?;

    lockfile.lock()?;
    io::stdin().read(&mut [0; 1])?;
    Ok(())
}
