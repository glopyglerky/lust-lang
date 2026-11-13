//! Port of execute/incremental/incremental.go (the buildinfo reader) and
//! execute/incremental/host.go (the mtime host).
//!
//! PORT: `ReadBuildInfoProgram` (incremental.go:43) is not here. It builds
//! a `Program` from `buildInfoToSnapshot`, so it lives with `Program` in
//! `program.rs`.

use super::build_info::*;
use crate::frontend::prelude::*;
use std::time::SystemTime;

// Go: incremental/incremental.go:9 BuildInfoReader

// Go: incremental/incremental.go:15 buildInfoReader
// PORT: Go unexported type behind the `BuildInfoReader` interface.


// Go: incremental/incremental.go:30 (json.Unmarshal into BuildInfo)
// PORT: split out of `ReadBuildInfo` so callers with the text in hand (and
// tests) can parse it. Returns `None` on any unmarshal error, like Go.

// Go: incremental/program.go:300 (json.Marshal of the BuildInfo)
// PORT: the text that Go writes for a `BuildInfo`. `snapshotToBuildInfo`
// callers use it to write the `.tsbuildinfo` file.

// Go: incremental/incremental.go:38 NewBuildInfoReader

// ---------------------------------------------------------------------------
// incremental/host.go
// ---------------------------------------------------------------------------

// Go: incremental/host.go:10 Host
// PORT: Go `time.Time` is `Option<SystemTime>`; `None` is the Go zero time
// (as in `vfs::FileInfo::mod_time`).
pub trait Host {
    fn fs(&self) -> Rc<dyn Fs>;
    fn get_m_time(&self, file_name: &str) -> Option<SystemTime>;
    fn set_m_time(&self, file_name: &str, m_time: Option<SystemTime>) -> Result<(), FsError>;
}

// Go: incremental/host.go:16 host
// PORT: Go unexported type behind the `Host` interface.


// Go: incremental/host.go:34 CreateHost

// Go: incremental/host.go:38 GetMTime
