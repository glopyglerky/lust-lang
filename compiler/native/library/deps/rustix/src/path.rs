// `rustix::path`: the argument trait of the path-taking calls.
// Upstream `Arg` hands the backend a C string; the adapter hands
// the sysx shims the path bytes (the C side adds the NUL). The
// fs calls reject a path with an interior NUL with EINVAL
// before the syscall, as upstream's C string conversion does.

use std::ffi::{CStr, OsStr, OsString};
use std::path::{Path, PathBuf};

/// A path argument. Upstream's methods (`as_str`, `into_with_c_str`,
/// ...) are not here; the one hook is a `__lust_` name user code cannot
/// call or define (check/platform.rs), so a user impl of `Arg` rejects.
pub trait Arg {
    /// The path bytes (no terminator; the shim adds it).
    fn __lust_path_bytes(&self) -> Vec<u8>;
}

impl Arg for &str {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

#[cfg(feature = "alloc")]
impl Arg for String {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

#[cfg(feature = "alloc")]
impl Arg for &String {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

#[cfg(feature = "std")]
impl Arg for &OsStr {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        self.as_encoded_bytes().to_vec()
    }
}

#[cfg(feature = "std")]
impl Arg for &OsString {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        self.as_encoded_bytes().to_vec()
    }
}

#[cfg(feature = "std")]
impl Arg for &Path {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        self.as_os_str().as_encoded_bytes().to_vec()
    }
}

#[cfg(feature = "std")]
impl Arg for &PathBuf {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        self.as_os_str().as_encoded_bytes().to_vec()
    }
}

#[cfg(feature = "std")]
impl Arg for PathBuf {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        self.as_os_str().as_encoded_bytes().to_vec()
    }
}

/// A C string path (upstream passes it to the syscall as it is; its
/// bytes have no interior NUL, so the EINVAL check never fires).
impl Arg for &CStr {
    fn __lust_path_bytes(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}
