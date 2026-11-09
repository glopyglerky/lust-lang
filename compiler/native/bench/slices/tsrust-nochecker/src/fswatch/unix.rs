//! The subset of Go `golang.org/x/sys/unix` (v0.46.0, linux) that fswatch
//! uses. There is no Go file for this module in typescript-go. Linux only:
//! the fswatch backends that use it (inotify, fanotify) are Linux only.
//!
//! PORT: D-W1 allows a safe syscall crate. The inotify, pipe, poll, read,
//! write, close, open, getdents, lstat and statfs calls go through `rustix`
//! (safe API) or `std`. rustix has no fanotify or `name_to_handle_at`, so
//! fanotify goes through `nix::sys::fanotify` and `name_to_handle_at` through
//! the `name-to-handle-at` crate (both safe APIs). No `libc`, no `unsafe`.
//! Every syscall keeps its Go name, parameters and result. Types, constants
//! and errno values are the Go values (zerrors_linux.go,
//! zerrors_linux_<arch>.go, ztypes_linux.go, ztypes_linux_<arch>.go). The
//! open(2) and inotify_init1(2) flags differ between targets (O_DIRECTORY
//! and O_NOFOLLOW on arm64 are the amd64 O_DIRECT and O_LARGEFILE bits), so
//! they are rustix's values for the target. A Go untyped constant gets the
//! Rust type of the place fswatch uses it. Go `int` is `i32`, Go `uint` is
//! `u32`. A syscall error is a `GoError` made from an `Errno`
//! (`errors::from_value`, see `fswatch::syscall`), so
//! `errors::is(&err, &errors::from_value(unix::EINTR))` works as in Go.
//! The `from_ne_bytes` readers replace fswatch's `unsafe.Pointer` casts of
//! kernel records. Those records (inotify_event, fanotify_event_metadata,
//! linux_dirent64) have one layout on every Linux target. `Stat_t` and
//! `Statfs_t` are filled from `std` and rustix, not cast, so the amd64
//! field types hold the values of any target. A path is a Go string in the
//! port form, so each call passes its Go bytes (`osvfs::os_path`).

use crate::fswatch::prelude::*;
pub use crate::fswatch::syscall::*;
use crate::gostd::errors;
use std::os::fd::AsFd;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// open(2) flags (Go int), the target's values.

// poll(2) (Go PollFd.Events is int16).
pub const POLLIN: i16 = 0x1;

// *at(2) (Go int).
pub const AT_FDCWD: i32 = -0x64;

// Dirent.Type values (Go uint8).
pub const DT_UNKNOWN: u8 = 0x0;
pub const DT_DIR: u8 = 0x4;

// Stat_t.Mode bits (Go uint32).
pub const S_IFMT: u32 = 0xf000;
pub const S_IFDIR: u32 = 0x4000;

// inotify_init1(2) flags (Go int), the target's values.

// The flags above against the Go values (x/sys v0.46.0 zerrors_linux.go,
// zerrors_linux_amd64.go and zerrors_linux_arm64.go). Only O_DIRECTORY and
// O_NOFOLLOW differ between the two.
#[cfg(target_arch = "aarch64")]
const _: () = assert!(O_DIRECTORY == 0x4000 && O_NOFOLLOW == 0x8000);

// inotify event and watch mask bits (Go uint32).
pub const IN_MODIFY: u32 = 0x2;
pub const IN_MOVED_FROM: u32 = 0x40;
pub const IN_MOVED_TO: u32 = 0x80;
pub const IN_CREATE: u32 = 0x100;
pub const IN_DELETE: u32 = 0x200;
pub const IN_DELETE_SELF: u32 = 0x400;
pub const IN_MOVE_SELF: u32 = 0x800;
pub const IN_Q_OVERFLOW: u32 = 0x4000;
pub const IN_ONLYDIR: u32 = 0x1000000;
pub const IN_DONT_FOLLOW: u32 = 0x2000000;
pub const IN_EXCL_UNLINK: u32 = 0x4000000;
pub const IN_ISDIR: u32 = 0x40000000;

// fanotify_init(2) flags (Go uint).
pub const FAN_CLASS_NOTIF: u32 = 0x0;
pub const FAN_CLOEXEC: u32 = 0x1;
pub const FAN_NONBLOCK: u32 = 0x2;
pub const FAN_REPORT_FID: u32 = 0x200;
pub const FAN_REPORT_DFID_NAME: u32 = 0xc00;

// fanotify_mark(2) flags (Go uint).
pub const FAN_MARK_ADD: u32 = 0x1;
pub const FAN_MARK_REMOVE: u32 = 0x2;
pub const FAN_MARK_DONT_FOLLOW: u32 = 0x4;
pub const FAN_MARK_ONLYDIR: u32 = 0x8;

// fanotify event mask bits (Go uint64).
pub const FAN_MODIFY: u64 = 0x2;
pub const FAN_MOVED_FROM: u64 = 0x40;
pub const FAN_MOVED_TO: u64 = 0x80;
pub const FAN_CREATE: u64 = 0x100;
pub const FAN_DELETE: u64 = 0x200;
pub const FAN_DELETE_SELF: u64 = 0x400;
pub const FAN_MOVE_SELF: u64 = 0x800;
pub const FAN_Q_OVERFLOW: u64 = 0x4000;
pub const FAN_EVENT_ON_CHILD: u64 = 0x8000000;
pub const FAN_RENAME: u64 = 0x10000000;
pub const FAN_ONDIR: u64 = 0x40000000;

// fanotify event records (Go uint8).
pub const FANOTIFY_METADATA_VERSION: u8 = 0x3;
pub const FAN_EVENT_INFO_TYPE_DFID_NAME: u8 = 0x2;
pub const FAN_EVENT_INFO_TYPE_DFID: u8 = 0x3;
pub const FAN_EVENT_INFO_TYPE_OLD_DFID_NAME: u8 = 0xa;
pub const FAN_EVENT_INFO_TYPE_NEW_DFID_NAME: u8 = 0xc;

// Go: ztypes_linux.go SizeofInotifyEvent
pub const SIZEOF_INOTIFY_EVENT: usize = 0x10;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

// Go: ztypes_linux.go PollFd
#[derive(Clone, Copy, Debug, Default)]
pub struct PollFd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

// Go: ztypes_linux.go InotifyEvent
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InotifyEvent {
    pub wd: i32,
    pub mask: u32,
    pub cookie: u32,
    pub len: u32,
}

impl InotifyEvent {
}

// Go: ztypes_linux.go FanotifyEventMetadata
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FanotifyEventMetadata {
    pub event_len: u32,
    pub vers: u8,
    pub reserved: u8,
    pub metadata_len: u16,
    pub mask: u64,
    pub fd: i32,
    pub pid: i32,
}

impl FanotifyEventMetadata {
}

// Go: ztypes_linux_amd64.go Dirent
/// PORT: the record header only. Go's `Name [256]int8` and padding are read
/// from the buffer at `DIRENT_NAME_OFFSET` (Go `unsafe.Offsetof(d.Name)`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Dirent {
    pub ino: u64,
    pub off: i64,
    pub reclen: u16,
    pub type_: u8,
}

/// Go `unsafe.Offsetof(unix.Dirent{}.Name)`: 19 on every Linux target
/// (linux_dirent64).
pub const DIRENT_NAME_OFFSET: usize = 19;

impl Dirent {
}

// PORT: the first N bytes of a kernel record, zero padded.

// Go: ztypes_linux_amd64.go Timespec
#[derive(Clone, Copy, Debug, Default)]
pub struct Timespec {
    pub sec: i64,
    pub nsec: i64,
}

// Go: ztypes_linux_amd64.go Stat_t
// PORT: Go's blank padding fields are left out.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Stat_t {
    pub dev: u64,
    pub ino: u64,
    pub nlink: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
    pub size: i64,
    pub blksize: i64,
    pub blocks: i64,
    pub atim: Timespec,
    pub mtim: Timespec,
    pub ctim: Timespec,
}

// Go: ztypes_linux.go Fsid
#[derive(Clone, Copy, Debug, Default)]
pub struct Fsid {
    pub val: [i32; 2],
}

// Go: ztypes_linux_amd64.go Statfs_t
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Statfs_t {
    pub type_: i64,
    pub bsize: i64,
    pub blocks: u64,
    pub bfree: u64,
    pub bavail: u64,
    pub files: u64,
    pub ffree: u64,
    pub fsid: Fsid,
    pub namelen: i64,
    pub frsize: i64,
    pub flags: i64,
    pub spare: [i64; 4],
}

// Go: syscall_linux.go:2343 FileHandle
/// FileHandle represents the C struct file_handle used by
/// name_to_handle_at (see NameToHandleAt) and open_by_handle_at (see
/// OpenByHandleAt).
///
/// PORT: Go keeps a `*fileHandle` header followed by the handle bytes in
/// one buffer. The port keeps the header fields and the bytes.
#[derive(Clone, Debug, Default)]
pub struct FileHandle {
    pub handle_type: i32,
    pub handle: Vec<u8>,
}

// Go: syscall_linux.go:2348 NewFileHandle
/// NewFileHandle constructs a FileHandle.
pub fn new_file_handle(handle_type: i32, handle: &[u8]) -> FileHandle {
    FileHandle {
        handle_type,
        handle: handle.to_vec(),
    }
}

impl FileHandle {
    // Go: syscall_linux.go:2358 FileHandle.Size
    pub fn size(&self) -> i32 {
        self.handle.len() as i32
    }

    // Go: syscall_linux.go:2359 FileHandle.Type
    pub fn type_(&self) -> i32 {
        self.handle_type
    }

    // Go: syscall_linux.go:2360 FileHandle.Bytes
    // PORT: Go returns nil for an empty handle; the port returns an empty
    // slice.
    pub fn bytes(&self) -> &[u8] {
        let n = self.size();
        if n == 0 {
            return &[];
        }
        &self.handle[..n as usize]
    }
}

// ---------------------------------------------------------------------------
// Syscalls
// ---------------------------------------------------------------------------

// PORT: Go passes raw fd numbers. Safe Rust cannot turn a number back into a
// file descriptor, so every fd that this shim opens stays in `FDS` under its
// number until `close`. A call that gets a number not in the table fails
// with EBADF, as the kernel does for a closed fd. `poll`, `read` and `write`
// hold their own reference, so a concurrent `close` takes effect when they
// return.

// PORT: an fd in `FDS`. A fanotify group keeps its `nix` `Fanotify`, because
// `fanotify_mark` needs the `Fanotify::mark` method and safe code cannot make
// a `Fanotify` again from an `OwnedFd`.









// Go: syscall_linux.go:139 Pipe2

// Go: syscall_linux.go:161 Poll

// Go: syscall_unix.go:166 Read

// Go: syscall_unix.go:179 Write

// Go: zsyscall_linux.go:615 Close

// Go: syscall_linux.go:117 Open

// Go: syscall_linux.go:123 Openat

// Go: readdirent_getdents.go:10 ReadDirent

// PORT: the `d_type` byte that rustix turned into a `FileType`.

// Go: syscall_linux_amd64.go:26 Lstat

// Go: zsyscall_linux.go:1077 InotifyInit1

// Go: zsyscall_linux.go:1061 InotifyAddWatch

// Go: zsyscall_linux.go:1088 InotifyRmWatch

// Go: zsyscall_linux.go:14 FanotifyInit

// Go: syscall_linux.go:53 FanotifyMark

// Go: syscall_linux.go:2370 NameToHandleAt

// Go: zsyscall_linux_amd64.go:357 Statfs
