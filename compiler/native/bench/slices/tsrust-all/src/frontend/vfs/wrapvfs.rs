//! Go: internal/vfs/wrapvfs/wrapvfs.go

use crate::frontend::prelude::*;
use std::time::SystemTime;

// Go: wrapvfs.go:9 Replacements
// PORT: nil Go funcs are `None`. `time.Time` is `Option<SystemTime>`
// (`None` is the Go zero time), as in the `Fs` trait.
#[derive(Default)]
pub struct Replacements {
    pub use_case_sensitive_file_names: Option<Box<dyn Fn() -> bool>>,
    pub file_exists: Option<Box<dyn Fn(&str) -> bool>>,
    pub read_file: Option<Box<dyn Fn(&str) -> (String, bool)>>,
    pub write_file: Option<Box<dyn Fn(&str, &str) -> Result<(), FsError>>>,
    pub append_file: Option<Box<dyn Fn(&str, &str) -> Result<(), FsError>>>,
    pub remove: Option<Box<dyn Fn(&str) -> Result<(), FsError>>>,
    pub chtimes:
        Option<Box<dyn Fn(&str, Option<SystemTime>, Option<SystemTime>) -> Result<(), FsError>>>,
    pub directory_exists: Option<Box<dyn Fn(&str) -> bool>>,
    pub get_accessible_entries: Option<Box<dyn Fn(&str) -> Entries>>,
    pub stat: Option<Box<dyn Fn(&str) -> Option<FileInfo>>>,
    pub realpath: Option<Box<dyn Fn(&str) -> String>>,
}

// Go: wrapvfs.go:23 Wrap
// PORT: the Go package function `wrapvfs.Wrap` is `wrapvfs_wrap`.

// Go: wrapvfs.go:30 wrappedFS

