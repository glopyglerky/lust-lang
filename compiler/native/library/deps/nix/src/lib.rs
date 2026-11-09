// Lust adapter for nix 0.31.3 (docs/ARCH.md section 3.6): the
// prctl THP forms ts-rust's thp_guard.rs uses, the fanotify
// group and mark forms fswatch/unix.rs uses (feature
// `fanotify`, tf-adapters F2), the `NixPath` trait of the mark
// call, the signal set of tsgo's start (sys::signal), the
// kqueue forms fswatch/unix_bsd.rs uses on Darwin (feature
// `event`, sys::event over sysx, with sys::time's TimeSpec),
// plus the Errno type they return. Everything else in
// nix is absent and rejects in lust (README).

pub mod errno;
pub mod sys;

// Upstream imports `Errno` privately at the root (`nix::errno::Errno`
// and the `nix::Error` alias are the public names).
use errno::Errno;

use std::ffi::{CStr, OsStr};
use std::path::{Path, PathBuf};

/// The nix error type (an alias of `Errno`, as upstream).
pub type Error = Errno;

/// A specialized `Result` type for nix APIs.
pub type Result<T> = core::result::Result<T, Errno>;

/// Common trait used to represent file system paths by many
/// Nix functions. The byte forms of the impls (str, OsStr,
/// [u8], Path, PathBuf) reject a path with an interior NUL
/// byte with EINVAL, as upstream's C string conversion does.
pub trait NixPath {
    /// Is the path empty?
    fn is_empty(&self) -> bool;
    /// Length of the path in bytes.
    fn len(&self) -> usize;
    /// Execute a function with this path as a `CStr`.
    fn with_nix_path<T, F: FnOnce(&CStr) -> T>(&self, f: F) -> Result<T>;
}

/// The `with_nix_path` of a byte form path.
fn bytes_with_nix_path<T, F: FnOnce(&CStr) -> T>(bytes: &[u8], f: F) -> Result<T> {
    if bytes.contains(&0) {
        return Err(Errno::EINVAL);
    }
    let mut buf: Vec<u8> = Vec::with_capacity(bytes.len() + 1);
    buf.extend_from_slice(bytes);
    buf.push(0);
    match CStr::from_bytes_with_nul(&buf) {
        Ok(s) => Ok(f(s)),
        Err(_) => Err(Errno::EINVAL),
    }
}

impl NixPath for [u8] {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn with_nix_path<T, F: FnOnce(&CStr) -> T>(&self, f: F) -> Result<T> {
        bytes_with_nix_path(self, f)
    }
}

impl NixPath for str {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn with_nix_path<T, F: FnOnce(&CStr) -> T>(&self, f: F) -> Result<T> {
        bytes_with_nix_path(self.as_bytes(), f)
    }
}

impl NixPath for OsStr {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn with_nix_path<T, F: FnOnce(&CStr) -> T>(&self, f: F) -> Result<T> {
        bytes_with_nix_path(self.as_encoded_bytes(), f)
    }
}

impl NixPath for CStr {
    fn is_empty(&self) -> bool {
        self.to_bytes().is_empty()
    }
    fn len(&self) -> usize {
        self.to_bytes().len()
    }
    fn with_nix_path<T, F: FnOnce(&CStr) -> T>(&self, f: F) -> Result<T> {
        Ok(f(self))
    }
}

impl NixPath for Path {
    fn is_empty(&self) -> bool {
        self.as_os_str().is_empty()
    }
    fn len(&self) -> usize {
        self.as_os_str().len()
    }
    fn with_nix_path<T, F: FnOnce(&CStr) -> T>(&self, f: F) -> Result<T> {
        bytes_with_nix_path(self.as_os_str().as_encoded_bytes(), f)
    }
}

impl NixPath for PathBuf {
    fn is_empty(&self) -> bool {
        self.as_os_str().is_empty()
    }
    fn len(&self) -> usize {
        self.as_os_str().len()
    }
    fn with_nix_path<T, F: FnOnce(&CStr) -> T>(&self, f: F) -> Result<T> {
        bytes_with_nix_path(self.as_os_str().as_encoded_bytes(), f)
    }
}
