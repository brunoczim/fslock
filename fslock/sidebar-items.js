initSidebarItems({"enum":[["EitherOsStr","Either borrowed or owned allocation of an OS-native string."]],"struct":[["LockFile","A handle to a file that is lockable. Does not delete the file. On both Unix and Windows, the lock is held by an individual handle, and not by the whole process. On Unix, however, under `fork` file descriptors might be duplicated sharing the same lock, but `fork` is usually `unsafe` in Rust."],["OsStr","Borrowed allocation of an OS-native string."],["OsString","Owned allocation of an OS-native string."]],"trait":[["IntoOsString","Conversion of anything into an owned OS-native string. If allocation fails, an error shall be returned."],["ToOsStr","Conversion of anything to an either borrowed or owned OS-native string. If allocation fails, an error shall be returned."]],"type":[["Error","An IO error."]]});