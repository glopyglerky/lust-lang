//! Go: internal/vfs/trackingvfs/trackingvfs.go
//!
//! Package trackingvfs provides a VFS wrapper that records every file path
//! accessed during compilation. This allows watch mode to know exactly which
//! files and directories the compiler depended on, including non-existent
//! paths from failed module resolution.

use crate::frontend::prelude::*;
use std::time::SystemTime;

// Go: trackingvfs.go:17 FS

// Go: trackingvfs.go:22 `var _ vfs.FS = (*FS)(nil)`
