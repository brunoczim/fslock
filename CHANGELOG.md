# 0.2.1
* Added `try_lock_with_pid` method.
* Corrected bug that would not seek lock files on UNIX (when writing PIDs), and
    so it would fill the previous bytes with nul-bytes.

# 0.2.0
* Writing PID on locked file via `lock_with_pid()` method.
* Unix and Windows locks are now always per-handle.
* Removed multilock feature as it became obsolete.

# 0.1.8
* Compiling on Android b32

# 0.1.7
* Support for locks per handle/fd via multilock feature

# 0.1.6
* Fixed #1: now fslock compiles on arm
