use fslock::LockFile;

fn main() -> Result<(), fslock::Error> {
    let mut lockfile = LockFile::open("trylock.test")?;

    if lockfile.try_lock()? {
        println!("SUCCESS");
    } else {
        println!("FAILURE");
    }

    Ok(())
}
