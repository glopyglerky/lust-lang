//! Go `syscall` RLIMIT_NOFILE handling (syscall/rlimit.go, go1.27.1). A Go
//! process raises its soft open-file limit to one below the hard limit at
//! start, and a process that it starts gets the original limit back
//! (syscall/exec_linux.go forkAndExecInChild1; `spawn` here).

#[cfg(unix)]
use std::sync::OnceLock;


// Go: rlimit.go:30 init

// Go: exec_linux.go:644 "Restore original rlimit." (go1.27.1)
