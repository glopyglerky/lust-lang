//! Go: internal/vfs/vfs.go and internal/vfs/internal/internal.go.
//!
//! The Go standard library types that `vfs.FS` uses (`io/fs.FileInfo`,
//! `io/fs.DirEntry`, `io/fs.FileMode`, `io/fs.WalkDirFunc`, the `io/fs`
//! error values and the `fs.Stat`, `fs.ReadDir` and `fs.ReadFile` helpers)
//! are ported here too, because Rust has no equivalent with the same
//! behavior.

use crate::frontend::prelude::*;
use std::io;
use std::time::SystemTime;

// Go: vfs.go:11 FS
// FS is a file system abstraction.
// PORT: Go interface values are shared pointers, so every method takes
// `&self`. Implementations that cache use interior mutability. Go
// `time.Time` parameters are `Option<SystemTime>`; `None` is the Go zero
// time.
pub trait Fs {
    // UseCaseSensitiveFileNames returns true if the file system is case-sensitive.
    fn use_case_sensitive_file_names(&self) -> bool;

    // FileExists returns true if the file exists.
    fn file_exists(&self, path: &str) -> bool;

    // ReadFile reads the file specified by path and returns the content.
    // If the file fails to be read, ok will be false.
    fn read_file(&self, path: &str) -> (String, bool);

    fn write_file(&self, path: &str, data: &str) -> Result<(), FsError>;

    // AppendFile appends data to the file at path, creating it if it does not exist.
    fn append_file(&self, path: &str, data: &str) -> Result<(), FsError>;

    // Removes `path` and all its contents. Will return the first error it encounters.
    fn remove(&self, path: &str) -> Result<(), FsError>;

    // Chtimes changes the access and modification times of the named
    fn chtimes(
        &self,
        path: &str,
        a_time: Option<SystemTime>,
        m_time: Option<SystemTime>,
    ) -> Result<(), FsError>;

    // DirectoryExists returns true if the path is a directory.
    fn directory_exists(&self, path: &str) -> bool;

    // GetAccessibleEntries returns the files/directories in the specified directory.
    // If any entry is a symlink, it will be followed.
    fn get_accessible_entries(&self, path: &str) -> Entries;

    // PORT: Go returns a nil `fs.FileInfo` interface when the path cannot be
    // read; that is `None`.
    fn stat(&self, path: &str) -> Option<FileInfo>;

    // Realpath returns the "real path" of the specified path,
    // following symlinks and correcting filename casing.
    fn realpath(&self, path: &str) -> String;

    /// PORT: Go type assertions of a `vfs.FS` value to the language service
    /// file system layers (ts#64291): the project package's
    /// `FileHandleSource`, `LayeredFileSystem`, `RebasableFileSystem`,
    /// `FileChangeExpander` and `*overlayFS`, and the api's
    /// `*requestFileSystem`. This crate (`goport_util`) cannot name those
    /// types, so a layer returns itself as `Any` and
    /// `project::as_fs_layer` downcasts it to its concrete type. Every other
    /// file system keeps this default (the assertion fails). Wrappers that
    /// forward to another `Fs` (`CachedFs`, `TrackingFs`, `wrapvfs`) must not
    /// forward it: Go asserts on the wrapper's own type.
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

// Go: vfs.go:47 Entries
#[derive(Clone, Debug, Default)]
pub struct Entries {
    pub files: Vec<String>,
    pub directories: Vec<String>,
    // Symlinks contains the names of entries in Files or Directories that were
    // originally symbolic links (or reparse points) on disk. The names are the
    // same as those in Files/Directories (i.e., the link name, not the target).
    // nil means symlink information is not available and the entries may need
    // to be re-checked for symlinks.
    // PORT: the Go nil map is `None`; an empty non-nil map is `Some(empty)`.
    pub symlinks: Option<FxHashSet<String>>,
}

// Go: io/fs/fs.go FileMode
// PORT: Go standard library type. Same bit values as Go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FileMode(pub u32);

impl FileMode {
    pub const DIR: FileMode = FileMode(1 << 31);
    pub const APPEND: FileMode = FileMode(1 << 30);
    pub const EXCLUSIVE: FileMode = FileMode(1 << 29);
    pub const TEMPORARY: FileMode = FileMode(1 << 28);
    pub const SYMLINK: FileMode = FileMode(1 << 27);
    pub const DEVICE: FileMode = FileMode(1 << 26);
    pub const NAMED_PIPE: FileMode = FileMode(1 << 25);
    pub const SOCKET: FileMode = FileMode(1 << 24);
    pub const SETUID: FileMode = FileMode(1 << 23);
    pub const SETGID: FileMode = FileMode(1 << 22);
    pub const CHAR_DEVICE: FileMode = FileMode(1 << 21);
    pub const STICKY: FileMode = FileMode(1 << 20);
    pub const IRREGULAR: FileMode = FileMode(1 << 19);
    pub const TYPE: FileMode = FileMode(
        Self::DIR.0
            | Self::SYMLINK.0
            | Self::NAMED_PIPE.0
            | Self::SOCKET.0
            | Self::DEVICE.0
            | Self::CHAR_DEVICE.0
            | Self::IRREGULAR.0,
    );
    pub const PERM: FileMode = FileMode(0o777);

    // Go: io/fs/fs.go FileMode.IsDir
    pub fn is_dir(self) -> bool {
        self.0 & Self::DIR.0 != 0
    }

    // Go: io/fs/fs.go FileMode.IsRegular
    pub fn is_regular(self) -> bool {
        self.0 & Self::TYPE.0 == 0
    }

    // Go: io/fs/fs.go FileMode.Perm
    pub fn perm(self) -> FileMode {
        FileMode(self.0 & Self::PERM.0)
    }

    // Go: io/fs/fs.go FileMode.Type
    pub fn type_(self) -> FileMode {
        FileMode(self.0 & Self::TYPE.0)
    }

    pub fn intersects(self, other: FileMode) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for FileMode {
    type Output = FileMode;
    fn bitor(self, rhs: FileMode) -> FileMode {
        FileMode(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for FileMode {
    fn bitor_assign(&mut self, rhs: FileMode) {
        self.0 |= rhs.0;
    }
}

// Go: vfs.go:63 FileInfo (= io/fs.FileInfo)
// PORT: the Go interface becomes a plain value. `Sys()` is not ported
// (nothing in scope reads it). `mod_time: None` is the Go zero time.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileInfo {
    pub name: String,
    pub size: i64,
    pub mode: FileMode,
    pub mod_time: Option<SystemTime>,
}

impl FileInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> i64 {
        self.size
    }

    pub fn mode(&self) -> FileMode {
        self.mode
    }

    pub fn mod_time(&self) -> Option<SystemTime> {
        self.mod_time
    }

    pub fn is_dir(&self) -> bool {
        self.mode.is_dir()
    }
}

// PORT: where `DirEntry.Info()` gets its data. Entries from `os.ReadDir`
// call `lstat` on the full path (`Lstat` holds the directory path, which
// the entries of one directory share, and `info` adds the name, as Go's
// `unixDirent` keeps `parent` and `name`); entries made from a `FileInfo`
// (`fs.FileInfoToDirEntry`, the bundled file system) return that value.
// `NotExist` is a `vfs.WalkDir` entry whose `Stat` found nothing (Go
// `walkDirEntry.Info` returns `ErrNotExist`, ts#64277).

// Go: vfs.go:60 DirEntry (= io/fs.DirEntry)


// Go: io/fs/readdir.go FileInfoToDirEntry
// PORT: Go returns nil for a nil info; callers here always have a value.

// Go: vfs.go:71 ErrInvalid, ErrPermission, ErrExist, ErrNotExist, ErrClosed,
// io/fs SkipAll, SkipDir (Go vfs.SkipAll and vfs.SkipDir until ts#64277)
// PORT: Go `error` values become one enum. `Path` is `*os.PathError`
// (the OS error is shared so the value can be cloned). `Other` is an
// `errors.New` message.
#[derive(Clone, Debug)]
pub enum FsError {
    Invalid,
    Permission,
    Exist,
    NotExist,
    Closed,
    SkipAll,
    SkipDir,
    Path {
        op: &'static str,
        path: String,
        err: Rc<io::Error>,
    },
    Other(String),
}

impl FsError {
    // PORT: builds a Go `*os.PathError`.
    pub fn path(op: &'static str, path: &str, err: io::Error) -> FsError {
        FsError::Path {
            op,
            path: path.to_string(),
            err: Rc::new(err),
        }
    }

    pub fn is_skip_all(&self) -> bool {
        matches!(self, FsError::SkipAll)
    }

    pub fn is_skip_dir(&self) -> bool {
        matches!(self, FsError::SkipDir)
    }
}

// Go: io/fs/walk.go WalkDirFunc (Go vfs.WalkDirFunc until ts#64277)
// PORT: Go passes a nil `DirEntry` and a nil `error` as `None`.

// PORT: the Go `io/fs.FS` value that `Common.RootFor` returns, with the
// `StatFS`, `ReadDirFS` and `ReadFileFS` methods that `fs.Stat`,
// `fs.ReadDir` and `fs.ReadFile` use. `os.DirFS` (osvfs.rs `DirFs`) is the
// only implementation in scope.

// Go: internal.go:15 Common
// PORT: Go func fields become fn pointers. A nil `IsReparsePoint` is `None`.

// Go: internal.go:20 RootLength

// Go: internal.go:30 SplitPath


// Go: internal.go:157 decodeBytes
// PORT: takes the bytes instead of a Go string that holds them. Go returns
// the bytes unchanged, so a Go string can hold invalid UTF-8. A Rust String
// cannot, so the text is the port form of the Go string
// (`scanner_util::go_string_from_bytes`, see `scanner_util::GO_STRING_MARKER`).

// Go: internal.go:175 decodeUtf16
// PORT: `order binary.ByteOrder` is `big_endian`. Go `binary.Read` reads
// len(s)/2 values and ignores an odd last byte; it cannot fail here.
// `utf16.Decode` replaces unpaired surrogates with U+FFFD, like
// `String::from_utf16_lossy`.

// Go: io/fs/fs.go ValidPath
// PORT: Go standard library, used by the `os.DirFS` port. `name` is a port
// form (see `scanner_util::GO_STRING_MARKER`); its Go bytes are valid UTF-8
// when it has no invalid byte or lone surrogate unit.

// Go: os/stat_linux.go fillFileStatFromSys
// PORT: Go standard library. Converts Rust metadata to the Go `FileInfo`
// with the same mode bits.

// Go: os/types_windows.go (*fileStat).mode (go1.27.1)
// PORT: off unix. Go reads the file attributes and the reparse tag; the port
// has the read-only bit, the link and the directory from `std::fs`, so a
// pipe, a device or a socket is a plain file here. Not run on such a target.
#[cfg(not(unix))]
pub fn file_info_from_metadata(name: &str, md: &std::fs::Metadata) -> FileInfo {
    let mut mode = FileMode(if md.permissions().readonly() {
        0o444
    } else {
        0o666
    });
    if md.file_type().is_symlink() {
        mode |= FileMode::SYMLINK;
    } else if md.is_dir() {
        mode |= FileMode::DIR | FileMode(0o111);
    }
    FileInfo {
        name: name.to_string(),
        size: md.len() as i64,
        mode,
        mod_time: md.modified().ok(),
    }
}

// Go: os/path_unix.go basename
// PORT: Go standard library. Removes trailing slashes and the leading
// directory name.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner_util::{GoUnit, go_string_bytes, go_unit_at};

    /// The units of a port form string.
    fn units(text: &str) -> Vec<GoUnit> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < text.len() {
            let (unit, size) = go_unit_at(text, i);
            out.push(unit);
            i += size;
        }
        out
    }

    // Each invalid byte is one unit, as Go reads one RuneError per byte. Real
    // U+EF80..U+EFFF, U+FDD0 and U+10F7xx chars stay chars. A file write
    // gives the source bytes back.
    #[test]
    fn invalid_utf8_bytes_round_trip() {
        let source: Vec<u8> = [
            &b"/\x80/u \xE2\x82A \xF0\x90\x80 \xFF\xE2\x82\xAC \xED\xA0\x80 "[..],
            "\u{EF80}\u{EFFF}\u{FDD0}\u{FDD0}\u{10F7FF}\u{10FFFF}\u{FDD0}".as_bytes(),
        ]
        .concat();
        let (text, ok) = decode_bytes(source.clone());
        assert!(ok);
        let units = units(&text);
        let invalid: Vec<u8> = units
            .iter()
            .filter_map(|unit| match unit {
                GoUnit::InvalidByte(b) => Some(*b),
                _ => None,
            })
            .collect();
        assert_eq!(
            invalid,
            [0x80, 0xE2, 0x82, 0xF0, 0x90, 0x80, 0xFF, 0xED, 0xA0, 0x80]
        );
        assert!(units.contains(&GoUnit::Char('\u{20AC}')));
        assert!(units.contains(&GoUnit::Char('\u{EF80}')));
        assert!(units.contains(&GoUnit::Char('\u{10F7FF}')));
        assert_eq!(
            units
                .iter()
                .filter(|&&unit| unit == GoUnit::Char('\u{FDD0}'))
                .count(),
            3
        );
        assert_eq!(&*go_string_bytes(&text), &source[..]);
        assert!(matches!(
            go_string_bytes("a\u{EF80}\u{EE00}b"),
            std::borrow::Cow::Borrowed(_)
        ));
    }
}
