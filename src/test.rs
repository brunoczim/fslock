use crate::{Error, LockFile};

#[cfg(feature = "std")]
#[test]
fn read_pid() -> Result<(), Error> {
    use std::fs::read_to_string;

    let path = "testfiles/read_pid.lock";
    let mut file = LockFile::open(path)?;
    file.lock_with_pid()?;

    let content_a = read_to_string(path)?;
    let content_b = read_to_string(path)?;

    assert!(content_a.trim().len() > 0);
    assert!(content_a.trim().chars().all(|ch| ch.is_ascii_digit()));

    assert_eq!(content_a, content_b);
    Ok(())
}

#[cfg(feature = "std")]
fn check_try_lock_example(
    lockpath: &str,
    expected: &[u8],
) -> Result<(), Error> {
    use std::process::{Command, Stdio};

    let child = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("--example")
        .arg("try_lock")
        .arg("--")
        .arg(lockpath)
        .stdout(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;

    assert!(output.status.success());
    assert_eq!(output.stderr, b"");
    assert_eq!(output.stdout, expected);

    Ok(())
}

#[cfg(feature = "std")]
#[test]
fn other_process() -> Result<(), Error> {
    let path = "testfiles/other_process.lock";
    let mut file = LockFile::open(path)?;
    file.lock()?;
    check_try_lock_example(path, b"FAILURE\n")?;
    file.unlock()?;
    check_try_lock_example(path, b"SUCCESS\n")?;
    Ok(())
}

#[cfg(feature = "std")]
#[test]
fn other_process_but_curr_reads() -> Result<(), Error> {
    use std::fs::read_to_string;

    let path = "testfiles/other_process_but_curr_reads.lock";
    let mut file = LockFile::open(path)?;
    file.lock()?;

    let mut _content = read_to_string(path)?;
    check_try_lock_example(path, b"FAILURE\n")?;

    file.unlock()?;
    check_try_lock_example(path, b"SUCCESS\n")?;
    Ok(())
}

#[cfg(all(feature = "std", any(not(unix), feature = "multilock")))]
#[test]
fn exclusive_lock_cases() -> Result<(), Error> {
    let path = "testfiles/exclusive_lock_cases.lock";

    let mut f1 = LockFile::open_excl(path)?;
    let mut f2 = LockFile::open_excl(path)?;

    // f1 will get the lock; f2 can't.
    assert!(f1.try_lock()?);
    assert!(!f2.try_lock()?);

    // have f2 wait for f1.
    let thr = std::thread::spawn(move || {
        f2.lock().unwrap();
        f2
    });

    // Sleep here a little, so that the other thread has time to
    // block on the fd-lock.
    std::thread::sleep(std::time::Duration::from_millis(100));
    drop(f1); // Causes f1 to unlock.

    let f2 = thr.join().unwrap();

    assert!(f2.owns_lock());

    Ok(())
}
