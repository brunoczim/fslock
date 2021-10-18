# 0.2.0
* Writing PID on locked file via `lock_with_pid()` method.
* Unix and Windows locks are now always per-handle.
* Removed multilock feature as it became obsolete.

# 0.1.7
* Multilock feature added (targeting Unix).

# 0.1.6
* Fixed #1: now fslock compiles on arm
