//! Go: internal/vfs/osvfs/os.go. The realpath and symlink helpers it calls
//! are in `frontend::nativepath`. Off Linux: the Windows branches of os.go
//! and the Windows `path/filepath` pieces (`FromSlash`, `Abs`, `Clean`) are
//! ported; the other targets use the unix ones.
//!
//! The Go standard library pieces that osvfs reaches (`os.DirFS`,
//! `os.RemoveAll`, `filepath.Abs`, `filepath.Clean`) are ported here too.

use crate::frontend::prelude::*;
use std::borrow::Cow;
use std::cell::OnceCell;
use std::ffi::{OsStr, OsString};
use std::io::{self, Write as _};
#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
use std::path::{Path as OsPath, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

// PORT: a Go path is a Go string, so it can hold any bytes, and Go passes
// those bytes to the OS unchanged. A port string is the port form of a Go
// string (see `scanner_util::GO_STRING_MARKER`). `os_path` gives the OS the
// Go bytes, and `go_string_from_os` turns OS bytes (file names, link
// targets, the working directory, arguments) into the port form. Every OS
// call in the port goes through them.



// PORT divergence: off unix an OS path is text (UTF-16 on Windows), and the
// port converts it lossily. Go on Windows uses WTF-8 (go1.27.1
// syscall/wtf8_windows.go): `UTF16ToString` keeps an unpaired surrogate as
// its 3-byte WTF-8 form, and `UTF16FromString` turns those 3 bytes back into
// the surrogate, so a name read from the OS goes back to the OS unchanged.
// The port makes an unpaired surrogate U+FFFD. For other bytes that are not
// UTF-8, Go makes each byte U+FFFD, and `from_utf8_lossy` makes each bad
// sequence U+FFFD. A port of Go's form would use `encode_wide` and
// `from_wide` (`std::os::windows::ffi`). Not run on such a target.
#[cfg(not(unix))]
pub fn os_path(path: &str) -> Cow<'_, OsPath> {
    match crate::scanner_util::go_string_bytes(path) {
        Cow::Borrowed(_) => Cow::Borrowed(OsPath::new(path)),
        Cow::Owned(bytes) => {
            Cow::Owned(PathBuf::from(String::from_utf8_lossy(&bytes).into_owned()))
        }
    }
}

#[cfg(not(unix))]
pub fn go_string_from_os(s: impl Into<OsString>) -> String {
    crate::scanner_util::go_string_from_utf8(s.into().to_string_lossy().into_owned())
}




// PORT: Go reads files and the current directory through `sys.FS()` and
// `sys.GetCurrentDirectory()`, so a Go test swaps the whole OS for its
// `TestSys`. The port's file system is `Rc`, so the parse, checker and emit
// threads cannot share `sys.FS()`: they call `osvfs_fs()` and
// `os_current_dir()` directly. A test process installs an `OsOverride` once
// at start, and those calls then reach the test file system and directory.
// Only test processes install it; a real run never does and keeps the OS
// behavior below unchanged.

/// The file system and current directory that replace the OS in a test
/// process (see `install_os_override`).
pub struct OsOverride {
    /// Makes the file system for one thread. `osvfs_fs` calls it once per
    /// thread. Each value must share one state (for example a map behind
    /// an `Arc<Mutex>`), so that all threads see the same files. It must
    /// not call `osvfs_fs` itself.
    pub fs: Arc<dyn Fn() -> Rc<dyn Fs> + Send + Sync>,
    /// The value of `os_current_dir`.
    pub current_directory: String,
}

static OS_OVERRIDE: OnceLock<OsOverride> = OnceLock::new();

/// Replaces the OS file system and current directory for the rest of the
/// process. Install it before the first `osvfs_fs` or `os_current_dir`
/// call. Panics when an override is already installed.
pub fn install_os_override(o: OsOverride) {
    assert!(
        OS_OVERRIDE.set(o).is_ok(),
        "osvfs: an OS override is already installed"
    );
}

/// True when `install_os_override` has run in this process.
pub fn os_override_installed() -> bool {
    OS_OVERRIDE.get().is_some()
}

// PORT: the Go semaphores `blockingOpSema`, `readSema` and `writeSema`
// (os.go:20) limit concurrent syscalls. The port is single-threaded, so they
// are not ported.

// Go: os.go:30 FS
// FS creates a new FS from the OS file system.
// PORT: the Go package function `osvfs.FS` is `osvfs_fs`. Go returns one
// package-level value; this returns a clone of one per-thread value. With
// an OS override installed (a test process), the per-thread value is the
// one that the override's `fs` makes on the first call on that thread.

// Go: os.go:138 isReparsePoint

// Go: os.go:41 osFS
pub struct OsFs {
    common: Common,
}

// Go: os.go:46 isFileSystemCaseSensitive
// We do this right at startup to minimize the chance that executable gets moved or deleted.
// PORT: Go computes this in package init. The port computes it on first use.

// Go: os.go:77 swapCase
// Convert all lowercase chars to uppercase, and vice-versa

// PORT: Go `unicode.ToUpper` uses the simple one-rune case mapping. Rust
// only has the full mapping; a mapping to more than one char is treated as
// no simple mapping.

// PORT: Go `unicode.ToLower`; see `simple_to_upper`.




/// PORT: not in Go (perf). Off Linux, `stat` of each path after an
/// `lstat`.
#[cfg(not(target_os = "linux"))]
fn mod_times<'a>(paths: impl IntoIterator<Item = &'a str>) -> Vec<Option<Option<SystemTime>>> {
    let fs = osvfs_fs();
    paths
        .into_iter()
        .map(|path| {
            if std::fs::symlink_metadata(os_path(path)).is_ok_and(|md| md.is_symlink()) {
                return None;
            }
            Some(fs.stat(path).and_then(|stat| stat.mod_time()))
        })
        .collect()
}

// PORT: the Go `flag int` of `writeFileWithFlag`. Go passes
// `O_WRONLY|O_CREATE|O_TRUNC` or `O_WRONLY|O_CREATE|O_APPEND`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum WriteFlag {
    Truncate,
    Append,
}

// Go: os.go:122 osFSRealpath

impl OsFs {
    // Go: os.go:142 writeFileWithFlag

    // Go: os.go:158 ensureDirectoryExists

    // Go: os.go:163 writeFileEnsuringDir
}

// Go: os.go:193 GetGlobalTypingsCacheLocation
// PORT: not ported. Only the language server (cmd/tsc/lsp.go) calls it.

// Go: os/file.go DirFS
// PORT: Go standard library. `os.DirFS(dir)` with the `Stat`, `ReadDir`
// and `ReadFile` methods that `io/fs` helpers use.

// Go: os/file.go dirFS
pub struct DirFs {
    dir: String,
}

impl DirFs {
    // Go: os/file.go dirFS.join
    // PORT: Go `filepathlite.Localize` on Unix is `fs.ValidPath` plus a
    // NUL byte check.
}


// Go: os/dir.go ReadDir
// PORT: Go standard library. Returns the entries sorted by file name. The
// entry type comes from the directory entry (d_type) and does not follow
// symlinks. Go skips an entry that is removed before its `lstat`; so does
// the port. Go returns the entries read before an error; the port returns
// only the error (callers here drop the entries on error).



/// The entries of `os_read_dir`, not sorted.
#[cfg(not(target_os = "linux"))]
fn read_dir_entries(dirname: &str, dir: &Rc<str>) -> io::Result<Vec<DirEntry>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(os_path(dirname))? {
        let entry = entry?;
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err),
        };
        let typ = if file_type.is_dir() {
            FileMode::DIR
        } else if file_type.is_symlink() {
            FileMode::SYMLINK
        } else if file_type.is_file() {
            FileMode(0)
        } else {
            special_file_mode(file_type)
        };
        entries.push(DirEntry {
            name: go_string_from_os(entry.file_name()),
            typ,
            info: DirEntryInfo::Lstat(Rc::clone(dir)),
        });
    }
    Ok(entries)
}

// PORT: the `os_read_dir` mode of an entry that is not a directory, a link
// or a regular file. Linux maps the `d_type` in `read_dir_entries`.
#[cfg(all(unix, not(target_os = "linux")))]
fn special_file_mode(file_type: std::fs::FileType) -> FileMode {
    if file_type.is_block_device() {
        FileMode::DEVICE
    } else if file_type.is_char_device() {
        FileMode::DEVICE | FileMode::CHAR_DEVICE
    } else if file_type.is_fifo() {
        FileMode::NAMED_PIPE
    } else if file_type.is_socket() {
        FileMode::SOCKET
    } else {
        FileMode::IRREGULAR
    }
}

// PORT: off unix, std names no other file type. Not run on such a target.
#[cfg(not(unix))]
fn special_file_mode(_: std::fs::FileType) -> FileMode {
    FileMode::IRREGULAR
}

// Go: os/removeall_at.go RemoveAll
// PORT: Go standard library. Removes path and any children. A missing
// path is not an error. Rust `remove_dir_all` does not follow symlinks,
// like Go.

// Go: os/path.go:19 MkdirAll (go1.27.1)
// MkdirAll creates a directory named path,
// along with any necessary parents, and returns nil,
// or else returns an error.
// PORT: Go standard library. Go `Stat`, `Mkdir` and `Lstat` are
// `std::fs::metadata`, `DirBuilder::create` and `std::fs::symlink_metadata`.
// Off unix `perm` does not apply, as in Go. pprof.rs uses it too.

// Go: syscall.ENOTDIR (go1.27.1 syscall/zerrors_linux_amd64.go; darwin and
// the BSDs have the same value). On Windows it is ERROR_PATH_NOT_FOUND
// (syscall/zerrors_windows.go).
#[cfg(not(windows))]
const ENOTDIR: i32 = 0x14;
#[cfg(windows)]
const ENOTDIR: i32 = 3;

// Go: os.IsPathSeparator ('/' and, on Windows, '\\')
fn is_path_separator(c: u8) -> bool {
    c == b'/' || (cfg!(windows) && c == b'\\')
}

// Go: len(filepathlite.VolumeName(path)). There is no volume name on unix.
#[cfg(not(windows))]
fn volume_name_len(_: &str) -> usize {
    0
}
#[cfg(windows)]
fn volume_name_len(path: &str) -> usize {
    filepath_volume_name_len(path.as_bytes())
}

// Go: os/path.go endsWithDot
fn ends_with_dot(path: &str) -> bool {
    if path == "." {
        return true;
    }
    let b = path.as_bytes();
    b.len() >= 2 && b[b.len() - 1] == b'.' && b[b.len() - 2] == b'/'
}

// Go: path/filepath/path.go FromSlash
/// FromSlash returns the result of replacing each slash ('/') character in
/// path with a separator character. Multiple slashes are replaced by
/// multiple separators. On unix it is the identity.
pub fn filepath_from_slash(path: &str) -> Cow<'_, str> {
    if cfg!(windows) && path.contains('/') {
        return Cow::Owned(path.replace('/', "\\"));
    }
    Cow::Borrowed(path)
}

// Go: path/filepath/path.go Abs (unix)
// PORT: Go standard library.

// Go: path/filepath/path_windows.go abs
// PORT: Go standard library. Go `syscall.FullPath` is GetFullPathNameW, which
// `std::path::absolute` calls on Windows.
#[cfg(windows)]
fn filepath_abs(path: &str) -> Result<String, FsError> {
    // syscall.FullPath returns an error on empty path, because it's not a valid path.
    // To implement Abs behavior of returning working directory on empty string input,
    // special-case empty path by changing it to "." path. See golang.org/issue/24441.
    let path = if path.is_empty() { "." } else { path };
    let full_path = std::path::absolute(os_path(path))
        .map_err(|err| FsError::path("GetFullPathName", path, err))?;
    Ok(filepath_clean(&go_string_from_os(full_path)))
}

// Go: path/filepath/path.go Clean (unix)
// PORT: Go standard library. Lexical cleanup only.
#[cfg(not(windows))]
pub fn filepath_clean(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let p = path.as_bytes();
    let rooted = p[0] == b'/';
    let n = p.len();

    // Invariants:
    //	reading from path; r is index of next byte to process.
    //	writing to out; w is index of next byte to write.
    //	dotdot is index in out where .. must stop, either because
    //		it is the leading slash or it is a leading ../../.. prefix.
    let mut out: Vec<u8> = Vec::with_capacity(n);
    let (mut r, mut dotdot) = (0usize, 0usize);
    if rooted {
        out.push(b'/');
        r = 1;
        dotdot = 1;
    }

    while r < n {
        if p[r] == b'/' {
            // empty path element
            r += 1;
        } else if p[r] == b'.' && (r + 1 == n || p[r + 1] == b'/') {
            // . element
            r += 1;
        } else if p[r] == b'.' && p[r + 1] == b'.' && (r + 2 == n || p[r + 2] == b'/') {
            // .. element: remove to last /
            r += 2;
            if out.len() > dotdot {
                // can backtrack
                let mut w = out.len() - 1;
                while w > dotdot && out[w] != b'/' {
                    w -= 1;
                }
                out.truncate(w);
            } else if !rooted {
                // cannot backtrack, but not rooted, so append .. element.
                if !out.is_empty() {
                    out.push(b'/');
                }
                out.extend_from_slice(b"..");
                dotdot = out.len();
            }
        } else {
            // real path element.
            // add slash if needed
            if (rooted && out.len() != 1) || (!rooted && !out.is_empty()) {
                out.push(b'/');
            }
            // copy element
            while r < n && p[r] != b'/' {
                out.push(p[r]);
                r += 1;
            }
        }
    }

    // Turn empty string into "."
    if out.is_empty() {
        return ".".to_string();
    }
    // The input is valid UTF-8 and the cuts are at ASCII '/' bytes.
    String::from_utf8(out)
        .unwrap_or_else(|err| String::from_utf8_lossy(err.as_bytes()).into_owned())
}

// Go: internal/filepathlite/path.go Clean (windows)
// PORT: Go standard library (go1.27 internal/filepathlite, path_windows.go
// for the volume name and postClean). Go's `lazybuf` is `out` plus
// `changed` (Go's `buf != nil`: the output is no longer a prefix of the
// input).
#[cfg(windows)]
pub fn filepath_clean(path: &str) -> String {
    const SEPARATOR: u8 = b'\\';
    let original_path = path;
    let vol_len = filepath_volume_name_len(path.as_bytes());
    let p = &path.as_bytes()[vol_len..];
    if p.is_empty() {
        let o = original_path.as_bytes();
        if vol_len > 1 && win_is_path_separator(o[0]) && win_is_path_separator(o[1]) {
            // should be UNC
            return original_path.replace('/', "\\");
        }
        return format!("{original_path}.");
    }
    let rooted = win_is_path_separator(p[0]);

    // Invariants:
    //	reading from path; r is index of next byte to process.
    //	writing to buf; w is index of next byte to write.
    //	dotdot is index in buf where .. must stop, either because
    //		it is the leading slash or it is a leading ../../.. prefix.
    let n = p.len();
    let mut out: Vec<u8> = Vec::with_capacity(n);
    let mut changed = false;
    let append = |out: &mut Vec<u8>, changed: &mut bool, c: u8| {
        if !*changed && (out.len() >= n || p[out.len()] != c) {
            *changed = true;
        }
        out.push(c);
    };
    let (mut r, mut dotdot) = (0usize, 0usize);
    if rooted {
        append(&mut out, &mut changed, SEPARATOR);
        r = 1;
        dotdot = 1;
    }

    while r < n {
        if win_is_path_separator(p[r]) {
            // empty path element
            r += 1;
        } else if p[r] == b'.' && (r + 1 == n || win_is_path_separator(p[r + 1])) {
            // . element
            r += 1;
        } else if p[r] == b'.'
            && p[r + 1] == b'.'
            && (r + 2 == n || win_is_path_separator(p[r + 2]))
        {
            // .. element: remove to last separator
            r += 2;
            if out.len() > dotdot {
                // can backtrack
                let mut w = out.len() - 1;
                while w > dotdot && !win_is_path_separator(out[w]) {
                    w -= 1;
                }
                out.truncate(w);
            } else if !rooted {
                // cannot backtrack, but not rooted, so append .. element.
                if !out.is_empty() {
                    append(&mut out, &mut changed, SEPARATOR);
                }
                append(&mut out, &mut changed, b'.');
                append(&mut out, &mut changed, b'.');
                dotdot = out.len();
            }
        } else {
            // real path element.
            // add slash if needed
            if (rooted && out.len() != 1) || (!rooted && !out.is_empty()) {
                append(&mut out, &mut changed, SEPARATOR);
            }
            // copy element
            while r < n && !win_is_path_separator(p[r]) {
                append(&mut out, &mut changed, p[r]);
                r += 1;
            }
        }
    }

    // Turn empty string into "."
    if out.is_empty() {
        append(&mut out, &mut changed, b'.');
    }

    // postClean: avoid creating absolute paths on Windows
    if vol_len == 0 && changed {
        // If a ':' appears in the path element at the start of a path,
        // insert a .\ at the beginning to avoid converting relative paths
        // like a/../c: into c:.
        let first = out
            .iter()
            .position(|&c| win_is_path_separator(c))
            .unwrap_or(out.len());
        if out[..first].contains(&b':') {
            out.splice(0..0, [b'.', SEPARATOR]);
        } else if out.len() >= 3
            && win_is_path_separator(out[0])
            && out[1] == b'?'
            && out[2] == b'?'
        {
            // If a path begins with \??\, insert a \. at the beginning
            // to avoid converting paths like \a\..\??\c:\x into \??\c:\x
            // (equivalent to c:\x).
            out.splice(0..0, [SEPARATOR, b'.']);
        }
    }

    let mut result = original_path.as_bytes()[..vol_len].to_vec();
    result.extend_from_slice(&out);
    // FromSlash. The cuts are at ASCII bytes, so the bytes stay UTF-8.
    String::from_utf8(result)
        .unwrap_or_else(|err| String::from_utf8_lossy(err.as_bytes()).into_owned())
        .replace('/', "\\")
}

// Go: internal/filepathlite/path_windows.go IsPathSeparator
#[cfg(windows)]
pub fn win_is_path_separator(c: u8) -> bool {
    c == b'\\' || c == b'/'
}

// Go: internal/filepathlite/path_windows.go volumeNameLen
#[cfg(windows)]
pub fn filepath_volume_name_len(path: &[u8]) -> usize {
    if path.len() >= 2 && path[1] == b':' {
        // Path starts with a drive letter.
        //
        // Not all Windows functions necessarily enforce the requirement that
        // drive letters be in the set A-Z, and we don't try to here.
        //
        // We don't handle the case of a path starting with a non-ASCII character,
        // in which case the "drive letter" might be multiple bytes long.
        return 2;
    }
    if path.is_empty() || !win_is_path_separator(path[0]) {
        // Path does not have a volume component.
        return 0;
    }
    if win_path_has_prefix_fold(path, br"\\.")
        || win_path_has_prefix_fold(path, br"\\?")
        || win_path_has_prefix_fold(path, br"\??")
    {
        // Path starts with a device prefix: \\.\ for Local Device paths,
        // or \\?\ or \??\ for Root Local Device paths.
        if path.len() == 3 {
            return 3; // exactly \\., \\?, or \??
        }
        if win_path_has_prefix_fold(&path[4..], b"UNC") {
            // We're going to treat the UNC host and share as part of the volume
            // prefix for historical reasons, but this isn't really principled;
            // Windows's own GetFullPathName will happily remove the first
            // component of the path in this space, converting
            // \\.\unc\a\b\..\c into \\.\unc\a\c.
            return win_valid_volume_name_len(path, win_unc_len(path, br"\\.\UNC\".len()));
        }
        // We treat the next component after the device prefix as
        // part of the volume name, which means Clean(`\\?\c:\`)
        // won't remove the trailing \. (See #64028.)
        return match win_cut_path(&path[4..]) {
            None => win_valid_volume_name_len(path, path.len()),
            Some((_, rest)) => win_valid_volume_name_len(path, path.len() - rest.len() - 1),
        };
    }
    if path.len() >= 2 && win_is_path_separator(path[1]) {
        // Path starts with \\, and is a UNC path.
        return win_valid_volume_name_len(path, win_unc_len(path, 2));
    }
    0
}

// Go: internal/filepathlite/path_windows.go validVolumeNameLen
#[cfg(windows)]
fn win_valid_volume_name_len(path: &[u8], n: usize) -> usize {
    let mut p = &path[..n];
    while !p.is_empty() {
        let (part, rest) = win_cut_path(p).unwrap_or((p, &[]));
        if part == b".." {
            return 0;
        }
        p = rest;
    }
    n
}

// Go: internal/filepathlite/path_windows.go pathHasPrefixFold
// pathHasPrefixFold tests whether the path s begins with prefix,
// ignoring case and treating all path separators as equivalent.
// If s is longer than prefix, then s[len(prefix)] must be a path separator.
#[cfg(windows)]
fn win_path_has_prefix_fold(s: &[u8], prefix: &[u8]) -> bool {
    if s.len() < prefix.len() {
        return false;
    }
    for i in 0..prefix.len() {
        if win_is_path_separator(prefix[i]) {
            if !win_is_path_separator(s[i]) {
                return false;
            }
        } else if prefix[i].to_ascii_uppercase() != s[i].to_ascii_uppercase() {
            return false;
        }
    }
    if s.len() > prefix.len() && !win_is_path_separator(s[prefix.len()]) {
        return false;
    }
    true
}

// Go: internal/filepathlite/path_windows.go uncLen
// uncLen returns the length of the volume prefix of a UNC path.
// prefixLen is the prefix prior to the start of the UNC host;
// for example, for "//host/share", the prefixLen is len("//")==2.
#[cfg(windows)]
fn win_unc_len(path: &[u8], prefix_len: usize) -> usize {
    let mut count = 0;
    for (i, &c) in path.iter().enumerate().skip(prefix_len) {
        if win_is_path_separator(c) {
            count += 1;
            if count == 2 {
                return i;
            }
        }
    }
    path.len()
}

// Go: internal/filepathlite/path_windows.go cutPath
// cutPath slices path around the first path separator.
// PORT: Go's `found == false` is `None`.
#[cfg(windows)]
fn win_cut_path(path: &[u8]) -> Option<(&[u8], &[u8])> {
    let i = path.iter().position(|&c| win_is_path_separator(c))?;
    Some((&path[..i], &path[i + 1..]))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    // PORT: not in Go. Go `os.Chtimes` (utimensat on the path) sets the
    // mtime of a file that its owner cannot read; so does `chtimes`. The
    // test skips when the file still opens (root, CAP_DAC_OVERRIDE): then a
    // `chtimes` that opens the file would pass too.
    #[test]
    fn chtimes_without_read_permission() {
        let dir = std::env::temp_dir().join(format!("ts_goport_chtimes_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("out.js");
        std::fs::write(&file, "x").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o000)).unwrap();
        if std::fs::File::open(&file).is_ok() {
            let _ = std::fs::remove_dir_all(&dir);
            eprintln!("skipped: a file without read permission opens here");
            return;
        }
        let m_time = SystemTime::UNIX_EPOCH + std::time::Duration::new(1_700_000_000, 5);
        let result = osvfs_fs().chtimes(file.to_str().unwrap(), None, Some(m_time));
        let modified = std::fs::symlink_metadata(&file)
            .unwrap()
            .modified()
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(result.is_ok());
        assert_eq!(modified, m_time);
    }

    // PORT: not in Go. `os_mod_times_of_non_links` gives None for a path
    // whose name is a link, and else the mtime that `stat` gives: files in
    // one directory and in another, a directory reached through a link and
    // through 30 links, a link to a file, a missing file, a missing
    // directory, a time before 1970, a path with no file name, names
    // through 30 directory links that are a chain of 30 file links (more
    // than 40 links) and one file link, and a file through 41 directory
    // links (ELOOP). The test keeps the name of the removed `os_mod_times`
    // (followups21), so the protected name stays.
    #[test]
    fn os_mod_times_matches_stat() {
        let dir = std::env::temp_dir().join(format!("ts_goport_mod_times_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("a/b")).unwrap();
        std::fs::write(dir.join("a/x.ts"), "x").unwrap();
        std::fs::write(dir.join("a/y.ts"), "y").unwrap();
        std::fs::write(dir.join("a/b/z.ts"), "z").unwrap();
        std::os::unix::fs::symlink(dir.join("a/b"), dir.join("a/l")).unwrap();
        std::os::unix::fs::symlink(dir.join("a/x.ts"), dir.join("a/lx.ts")).unwrap();
        // Relative targets, so each link is one link whatever the temp dir.
        std::os::unix::fs::symlink("a", dir.join("d0")).unwrap();
        std::os::unix::fs::symlink("x.ts", dir.join("a/f0.ts")).unwrap();
        for i in 1..30 {
            std::os::unix::fs::symlink(format!("d{}", i - 1), dir.join(format!("d{i}"))).unwrap();
            std::os::unix::fs::symlink(format!("f{}.ts", i - 1), dir.join(format!("a/f{i}.ts")))
                .unwrap();
        }
        for i in 30..41 {
            std::os::unix::fs::symlink(format!("d{}", i - 1), dir.join(format!("d{i}"))).unwrap();
        }
        let old = SystemTime::UNIX_EPOCH - std::time::Duration::new(1000, 0)
            + std::time::Duration::new(0, 7);
        osvfs_fs()
            .chtimes(dir.join("a/y.ts").to_str().unwrap(), None, Some(old))
            .unwrap();
        let root = dir.to_str().unwrap();
        let paths: Vec<String> = [
            "a/x.ts",
            "a/y.ts",
            "a/lx.ts",
            "a/b/z.ts",
            "a/l/z.ts",
            "a/x.ts",
            "a/no.ts",
            "no/x.ts",
            "a/b/",
            "a",
            "d29/f29.ts",
            "d29/f0.ts",
            "d29/x.ts",
            "d40/x.ts",
        ]
        .iter()
        .map(|name| format!("{root}/{name}"))
        .collect();
        let m_times = os_mod_times_of_non_links(paths.iter().map(String::as_str));
        let fs = osvfs_fs();
        let want: Vec<Option<Option<SystemTime>>> = paths
            .iter()
            .map(|path| {
                let link = std::fs::symlink_metadata(path).is_ok_and(|md| md.is_symlink());
                (!link).then(|| fs.stat(path).and_then(|stat| stat.mod_time()))
            })
            .collect();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(m_times, want);
        assert_eq!(want[1], Some(Some(old)));
        assert_eq!(want[2], None);
        assert!(want[4].is_some_and(|m_time| m_time.is_some()));
        assert!(want[6] == Some(None) && want[7] == Some(None));
        assert!(want[10].is_none() && want[11].is_none());
        assert!(want[12].is_some_and(|m_time| m_time.is_some()));
        assert_eq!(want[12], want[0]);
        assert_eq!(want[13], Some(None));
    }

    // PORT: not in Go. Go `os.ReadDir` lstats an entry whose `d_type` is
    // unknown (os/file_unix.go:469 newUnixDirent), skips it when the lstat
    // finds no file and fails on any other lstat error (os/dir_unix.go:141).
    // Some file systems give no `d_type`, so the test makes every type
    // unknown (`read_dir_entries_typed`): the entries and their types equal
    // those of the `d_type` read (a directory, a file, a link to the
    // directory, a FIFO), an entry removed before its lstat is skipped, and
    // a directory with no search permission is an error. A probe lstat
    // there picks what the last part expects: the error when the probe
    // fails (a user), else x.ts as a file (root, CAP_DAC_OVERRIDE). The
    // probe, not the read's result, picks it, so a read that turns the
    // error into an entry fails.
    #[cfg(target_os = "linux")]
    #[test]
    fn read_dir_lstats_entries_of_unknown_type() {
        use rustix::fs::FileType;
        let dir = std::env::temp_dir().join(format!("ts_goport_dirent_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.ts"), "a").unwrap();
        std::fs::write(dir.join("gone.ts"), "g").unwrap();
        std::os::unix::fs::symlink("sub", dir.join("link")).unwrap();
        rustix::fs::mknodat(
            rustix::fs::CWD,
            dir.join("fifo"),
            FileType::Fifo,
            rustix::fs::Mode::from_raw_mode(0o644),
            0,
        )
        .unwrap();
        let root = dir.to_str().unwrap();
        let names = |entries: io::Result<Vec<DirEntry>>| -> Vec<(String, FileMode)> {
            let mut names: Vec<_> = entries
                .unwrap()
                .into_iter()
                .map(|entry| (entry.name, entry.typ))
                .collect();
            names.sort_by(|a, b| a.0.cmp(&b.0));
            names
        };
        let from_d_type = names(read_dir_entries(root, &Rc::from(root)));
        let mut seen = Vec::new();
        let from_lstat = names(read_dir_entries_typed(root, &Rc::from(root), |name, _| {
            seen.push(name.to_bytes().to_vec());
            if name.to_bytes() == b"gone.ts" {
                std::fs::remove_file(dir.join("gone.ts")).unwrap();
            }
            FileType::Unknown
        }));

        let locked = dir.join("sub");
        std::fs::write(locked.join("x.ts"), "x").unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o444)).unwrap();
        let locked_name = locked.to_str().unwrap();
        let probe = std::fs::symlink_metadata(locked.join("x.ts")).map(|_| ());
        let locked_d_type = read_dir_entries(locked_name, &Rc::from(locked_name)).map(|e| e.len());
        let locked_lstat = read_dir_entries_typed(locked_name, &Rc::from(locked_name), |_, _| {
            FileType::Unknown
        });
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
        let _ = std::fs::remove_dir_all(&dir);

        let want = [
            ("a.ts", FileMode(0)),
            ("fifo", FileMode::NAMED_PIPE),
            ("gone.ts", FileMode(0)),
            ("link", FileMode::SYMLINK),
            ("sub", FileMode::DIR),
        ]
        .map(|(name, typ)| (name.to_string(), typ));
        assert_eq!(from_d_type, want);
        assert_eq!(seen.len(), 5, "each entry once");
        let mut want_lstat = want.to_vec();
        want_lstat.retain(|(name, _)| name != "gone.ts");
        assert_eq!(from_lstat, want_lstat);
        assert_eq!(locked_d_type.ok(), Some(1));
        match probe {
            Err(err) => {
                assert_eq!(err.kind(), io::ErrorKind::PermissionDenied, "the probe");
                assert_eq!(
                    locked_lstat.map(|_| ()).map_err(|err| err.kind()),
                    Err(io::ErrorKind::PermissionDenied),
                    "the read fails as the lstat does"
                );
            }
            Ok(()) => {
                assert_eq!(
                    names(locked_lstat),
                    [("x.ts".to_string(), FileMode(0))],
                    "an lstat that works there lists x.ts"
                );
                eprintln!(
                    "skipped the error part: the lstat works in a directory with no search permission"
                );
            }
        }
    }
}
