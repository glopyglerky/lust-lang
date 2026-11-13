// Lust adapter for same-file 1.0.6 (docs/ARCH.md section 3.6):
// `is_same_file` and `Handle` over the file identity Unix std
// reports (`std::os::unix::fs::MetadataExt` dev and ino), the
// same calls in the same order as upstream's unix.rs, so the
// answers and the `io::Error` values are the real crate's.
// Upstream opens the path read-only and stats the open file;
// so does this adapter, and a `Handle` keeps the file open
// until it drops, as upstream.
//
// Not here (see README.md): `Handle::stdin`, `stdout`, and
// `stderr` (they need a File built from a raw fd without
// owning it, which lust std has no form for yet), and the
// `AsRawFd` and `IntoRawFd` impls for `Handle` (lust std has
// no `std::os::fd` traits yet). Code that names them rejects
// in lust, never answers differently. Layout queries on
// `Handle` reject (no exact-layout entry): upstream's private
// fields are mirrored for the Debug form, but their size is
// not promised.

use std::fs::File;
use std::io;
use std::path::Path;

/// The Unix handle: the open file plus its device and inode
/// numbers, with upstream's field order and Debug form.
mod imp {
    use std::fs::{File, OpenOptions};
    use std::hash::{Hash, Hasher};
    use std::io;
    use std::os::unix::fs::MetadataExt;
    use std::path::Path;

    #[derive(Debug)]
    pub struct Handle {
        file: Option<File>,
        // Upstream does not close std stream fds on drop. This
        // adapter has no std stream handles, so it is always
        // false; the field stays for the Debug form.
        is_std: bool,
        dev: u64,
        ino: u64,
    }

    impl Eq for Handle {}

    impl PartialEq for Handle {
        fn eq(&self, other: &Handle) -> bool {
            (self.dev, self.ino) == (other.dev, other.ino)
        }
    }

    impl Hash for Handle {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.dev.hash(state);
            self.ino.hash(state);
        }
    }

    impl Handle {
        pub fn from_path<P: AsRef<Path>>(p: P) -> io::Result<Handle> {
            Handle::from_file(OpenOptions::new().read(true).open(p)?)
        }

        pub fn from_file(file: File) -> io::Result<Handle> {
            let md = file.metadata()?;
            Ok(Handle { file: Some(file), is_std: false, dev: md.dev(), ino: md.ino() })
        }

        pub fn as_file(&self) -> &File {
            self.file.as_ref().unwrap()
        }

        pub fn as_file_mut(&mut self) -> &mut File {
            self.file.as_mut().unwrap()
        }

        pub fn dev(&self) -> u64 {
            self.dev
        }

        pub fn ino(&self) -> u64 {
            self.ino
        }
    }
}

/// A handle to a file that can be tested for equality with other
/// handles. Two handles compare equal when their device and
/// inode numbers are equal. A handle keeps its file open.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Handle(imp::Handle);

impl Handle {
    /// Construct a handle from a path. The file is opened in
    /// read-only mode, as upstream.
    pub fn from_path<P: AsRef<Path>>(p: P) -> io::Result<Handle> {
        imp::Handle::from_path(p).map(Handle)
    }

    /// Construct a handle from a file.
    pub fn from_file(file: File) -> io::Result<Handle> {
        imp::Handle::from_file(file).map(Handle)
    }

    /// Return a reference to the underlying file.
    pub fn as_file(&self) -> &File {
        self.0.as_file()
    }

    /// Return a mutable reference to the underlying file.
    pub fn as_file_mut(&mut self) -> &mut File {
        self.0.as_file_mut()
    }

    /// Return the underlying device number of this handle.
    pub fn dev(&self) -> u64 {
        self.0.dev()
    }

    /// Return the underlying inode number of this handle.
    pub fn ino(&self) -> u64 {
        self.0.ino()
    }
}

/// Returns true if the two file paths may correspond to the same
/// file. An error is returned when either path cannot be opened,
/// the first path first, as upstream.
pub fn is_same_file<P, Q>(path1: P, path2: Q) -> io::Result<bool>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    Ok(Handle::from_path(path1)? == Handle::from_path(path2)?)
}
