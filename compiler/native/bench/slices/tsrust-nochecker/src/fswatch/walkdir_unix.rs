//! Go: internal/fswatch/walkdir_unix.go and walkdir_dirent_linux.go (the
//! getdents directory walk used by the Linux backends and, on darwin and the
//! BSDs, by the kqueue backend).
//!
//! PORT: the syscalls go through the `unix` shim (rustix, D-W1). Dirent
//! records are read with `unix::Dirent::from_ne_bytes`
//! instead of Go's `unsafe.Pointer` cast. On darwin and the BSDs the shim
//! (unix_bsd.rs) writes the entries in the Linux record layout, so the
//! Linux helpers below serve there too; the darwin and BSD dirent helpers
//! (walkdir_dirent_{darwin,fileno,noreclen}.go) are not needed.

use crate::fswatch::prelude::*;

use crate::fswatch::unix;
use crate::gostd::errors;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

// Go: walkdir_unix.go:14 walkState
/// walkState carries state shared across the whole walk so we only
/// allocate one read buffer per top-level walkDir, not one per directory.
pub struct WalkState {
    pub buf: Vec<u8>,
}

// Go: walkdir_unix.go:21 walkDir

// Go: walkdir_unix.go:44 iterateDir

// Go: walkdir_unix.go:99 unixDirent
#[derive(Clone, Debug)]
pub struct UnixDirent {
    pub name: String,
    pub typ: u8,
}

// Go: walkdir_unix.go:108 readDirEntries

// Go: walkdir_dirent_linux.go:7 reclenOf
pub fn reclen_of(d: &unix::Dirent) -> u16 {
    d.reclen
}

// Go: walkdir_dirent_linux.go:8 inoOf
pub fn ino_of(d: &unix::Dirent) -> u64 {
    d.ino
}

// macOS file systems reject a name that is not UTF-8 (EILSEQ), so the test
// cannot make its directory there.
#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;

    // PORT: not in Go. A directory name that is not UTF-8 keeps its bytes:
    // the walk opens it and gives its entries, and each path names the file
    // on disk.
    #[test]
    fn walk_dir_keeps_non_utf8_names() {
        let dir = std::env::temp_dir().join(format!("ts_goport_walkdir_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let bad = dir.join(std::ffi::OsString::from_vec(b"d\xfe".to_vec()));
        std::fs::create_dir_all(&bad).unwrap();
        std::fs::write(bad.join("q.ts"), "").unwrap();
        let root = go_string_from_os(dir.clone().into_os_string());
        let mut seen: Vec<(std::path::PathBuf, bool)> = Vec::new();
        let result = walk_dir(
            &root,
            true,
            Some(&mut |path: &str, is_dir: bool| -> Result<(), GoError> {
                seen.push((
                    crate::frontend::vfs::osvfs::os_path(path).into_owned(),
                    is_dir,
                ));
                Ok(())
            }),
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.is_ok());
        assert_eq!(
            seen,
            vec![(dir, true), (bad.clone(), true), (bad.join("q.ts"), false)]
        );
    }
}
