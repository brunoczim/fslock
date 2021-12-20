# 0.2.1
* Add `try_lock_with_pid` method.

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
