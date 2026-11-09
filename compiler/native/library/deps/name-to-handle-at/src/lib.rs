// Lust adapter for name-to-handle-at 0.1.0 (docs/ARCH.md
// section 3.6): the two syscall wrappers, over the sysx
// intrinsics (src/runtime/sysx.c), which run the real crate's
// sizing loop (a zero-capacity probe, then a retry while the
// kernel answers EOVERFLOW with a larger size).
//
// Same results as the real crate: the path bytes pass to the
// kernel with one NUL appended (an interior NUL truncates the
// path there, exactly as the real crate's bytes do), errors
// are `io::Error` from the raw errno, and the mount id is
// `Unique` exactly when the AT_HANDLE_MNT_ID_UNIQUE flag bit
// is set. The real crate caches "syscall unsupported" after
// an ENOSYS; the kernel answers ENOSYS on every call there,
// so the observable result is the same without the cache.

use std::io;
use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};
use std::path::Path;

/// Follow symbolic links (`AT_SYMLINK_FOLLOW`).
pub const AT_SYMLINK_FOLLOW: i32 = 0x400;
/// Allow an empty path to name the fd itself (`AT_EMPTY_PATH`).
pub const AT_EMPTY_PATH: i32 = 0x1000;
/// Return a file identifier, not a handle (`AT_HANDLE_FID`).
pub const AT_HANDLE_FID: i32 = 0x200;
/// Return a unique mount id (`AT_HANDLE_MNT_ID_UNIQUE`).
pub const AT_HANDLE_MNT_ID_UNIQUE: i32 = 0x001;

/// A mount ID identifying the filesystem containing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountId {
    /// A traditional 32-bit mount ID that may be reused.
    Reusable(u32),
    /// A unique 64-bit mount ID (Linux 6.8+).
    Unique(u64),
}

/// A file handle that uniquely identifies a file within a filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHandle {
    pub handle_type: i32,
    pub handle: Vec<u8>,
}

/// Little-endian u64 field `i` of a sysx record.
fn rec_u64(rec: &[u8], i: usize) -> u64 {
    let mut v = 0u64;
    let base = i * 8;
    for k in 0..8 {
        v |= (rec[base + k] as u64) << (8 * k);
    }
    v
}

/// Obtains a file handle and mount ID for a file
/// (`name_to_handle_at(2)`).
pub fn name_to_handle_at<Fd: AsFd>(fd: &Fd, path: &Path, flags: i32) -> io::Result<(FileHandle, MountId)> {
    let bytes = path.as_os_str().as_encoded_bytes();
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_name_to_handle_at(fd.as_fd().as_raw_fd(), bytes, flags, &mut rec);
    if e != 0 {
        return Err(io::Error::from_raw_os_error(e));
    }
    let handle = FileHandle { handle_type: rec_u64(&rec, 0) as u32 as i32, handle: rec[16..].to_vec() };
    let mount_id = rec_u64(&rec, 1);
    let mount_id = if flags & AT_HANDLE_MNT_ID_UNIQUE != 0 { MountId::Unique(mount_id) } else { MountId::Reusable(mount_id as u32) };
    Ok((handle, mount_id))
}

/// Opens a file using a file handle (`open_by_handle_at(2)`).
/// A handle longer than a u16 count is EINVAL, as in the real
/// crate (its syscall buffer size field conversion).
pub fn open_by_handle_at<Fd: AsFd>(mount_fd: &Fd, handle: &FileHandle, flags: i32) -> io::Result<OwnedFd> {
    if handle.handle.len() > 65535 {
        return Err(io::Error::from_raw_os_error(22));
    }
    let fd = core::intrinsics::sys_x_open_by_handle_at(mount_fd.as_fd().as_raw_fd(), handle.handle_type, &handle.handle, flags);
    if fd < 0 {
        Err(io::Error::from_raw_os_error(-fd))
    } else {
        Ok(OwnedFd::from_raw_fd(fd))
    }
}
