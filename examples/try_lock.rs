use fslock::LockFile;
use std::{env, process};

fn main() -> Result<(), fslock::Error> {
    let mut args = env::args();
    args.next();

    let path = match args.next() {
        Some(arg) if args.next().is_none() => arg,
        _ => {
            eprintln!("Expected one argument");
            process::exit(1);
        },
    };

    let mut lockfile = LockFile::open(&path)?;

    if lockfile.try_lock()? {
        println!("SUCCESS");
    } else {
        println!("FAILURE");
    }

    Ok(())
}
