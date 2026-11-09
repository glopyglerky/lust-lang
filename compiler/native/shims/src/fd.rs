//! Files, pipes, poll, terminal size, and file watching.
//!
//! This mirrors ts-rust fswatch/unix.rs: an fd this shim opens stays in
//! FDS under its raw number until lust_shim_close. A fanotify group
//! keeps its nix Fanotify value because fanotify_mark needs it. Calls
//! that read an fd hold an Arc for the call, so a close on another
//! thread takes effect when the call returns.
//!
//! Fds the shim does not own (stdio, C-created) can be read, written,
//! polled, fstat-ed, and sized through a borrowed raw fd. The shim
//! never closes those.

use std::collections::HashMap;
use std::ffi::{c_int, c_void};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::sync::{Arc, LazyLock, Mutex};

use crate::{OK, ShimBuf, ShimErr, ShimResult, buf_put, bytes, bytes_mut, guard, os_path, put, status};

/// An fd the shim owns.
enum ShimFd {
    Owned(OwnedFd),
    Fanotify(nix::sys::fanotify::Fanotify),
}

impl AsFd for ShimFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        match self {
            ShimFd::Owned(fd) => fd.as_fd(),
            ShimFd::Fanotify(f) => f.as_fd(),
        }
    }
}

static FDS: LazyLock<Mutex<HashMap<i32, Arc<ShimFd>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

const AT_FDCWD: i32 = -100;
const EBADF_RAW: i32 = 9;
const EINVAL_RAW: i32 = 22;

/// Puts an owned fd in the table and returns its raw number.
fn register(fd: ShimFd) -> i32 {
    use std::os::fd::AsRawFd;
    let n = fd.as_fd().as_raw_fd();
    FDS.lock().unwrap_or_else(|p| p.into_inner()).insert(n, Arc::new(fd));
    n
}

/// The shim-owned fd for a raw number, or EBADF.
fn lookup(fd: i32) -> ShimResult<Arc<ShimFd>> {
    FDS.lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&fd)
        .cloned()
        .ok_or_else(|| ShimErr::errno(EBADF_RAW, "bad file descriptor (not a shim fd)"))
}

/// How a call sees an fd: shim-owned, or a borrowed raw number.
enum FdRef {
    Owned(Arc<ShimFd>),
    Raw(i32),
}

impl FdRef {
    /// A borrowed view for rustix calls. Sound for a table entry. For a
    /// raw number this uses BorrowedFd::borrow_raw, so the caller must
    /// keep that fd open for the call, as with any raw fd API.
    fn borrowed(&self) -> BorrowedFd<'_> {
        match self {
            FdRef::Owned(f) => f.as_fd(),
            FdRef::Raw(n) => unsafe { BorrowedFd::borrow_raw(*n) },
        }
    }
}

impl AsFd for FdRef {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.borrowed()
    }
}

/// Table first, else a borrowed raw fd. Negative numbers are invalid
/// here; AT_FDCWD is handled by the *at calls before this.
fn resolve(fd: i32) -> ShimResult<FdRef> {
    if fd < 0 {
        return Err(ShimErr::invalid("fd"));
    }
    Ok(match FDS.lock().unwrap_or_else(|p| p.into_inner()).get(&fd).cloned() {
        Some(f) => FdRef::Owned(f),
        None => FdRef::Raw(fd),
    })
}

/// The C record shapes. They must match lust_shim.h.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CStat {
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
    pub atime_sec: i64,
    pub atime_nsec: i64,
    pub mtime_sec: i64,
    pub mtime_nsec: i64,
    pub ctime_sec: i64,
    pub ctime_nsec: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CStatx {
    pub mask: u32,
    pub mode: u32,
    pub mtime_sec: i64,
    pub mtime_nsec: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CStatvfs {
    pub bsize: u64,
    pub blocks: u64,
    pub bfree: u64,
    pub bavail: u64,
    pub files: u64,
    pub ffree: u64,
    pub fsid: u64,
    pub namemax: u64,
    pub frsize: u64,
    pub flag: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CPollFd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CWinsize {
    pub row: u16,
    pub col: u16,
    pub xpixel: u16,
    pub ypixel: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CTimestamps {
    pub atime_sec: i64,
    pub atime_nsec: i64,
    pub mtime_sec: i64,
    pub mtime_nsec: i64,
}

/// A rustix Stat as the C record. Field widths vary by target in
/// rustix, so the casts stay even where one is a no-op here.
#[allow(clippy::unnecessary_cast)]
fn to_cstat(s: &rustix::fs::Stat) -> CStat {
    CStat {
        dev: s.st_dev,
        ino: s.st_ino,
        nlink: s.st_nlink,
        mode: s.st_mode,
        uid: s.st_uid,
        gid: s.st_gid,
        rdev: s.st_rdev,
        size: s.st_size,
        blksize: s.st_blksize as i64,
        blocks: s.st_blocks,
        atime_sec: s.st_atime,
        atime_nsec: s.st_atime_nsec as i64,
        mtime_sec: s.st_mtime,
        mtime_nsec: s.st_mtime_nsec as i64,
        ctime_sec: s.st_ctime,
        ctime_nsec: s.st_ctime_nsec as i64,
    }
}

// ---------------------------------------------------------------------
// Pipes and byte IO
// ---------------------------------------------------------------------

/// pipe2 with raw PipeFlags bits (rustix pipe_with).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_pipe2(flags: u32, out_fds: *mut i32) -> c_int {
    match guard(|| {
        if out_fds.is_null() {
            return Err(ShimErr::invalid("out_fds"));
        }
        let flags = rustix::pipe::PipeFlags::from_bits_retain(flags);
        let (r, w) = rustix::pipe::pipe_with(flags)?;
        Ok((register(ShimFd::Owned(r)), register(ShimFd::Owned(w))))
    }) {
        Ok((r, w)) => {
            unsafe {
                *out_fds = r;
                *out_fds.add(1) = w;
            }
            OK
        }
        Err(s) => s,
    }
}

/// One read call. The caller loops and retries EINTR, as ts-rust does.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_read(fd: i32, buf: *mut u8, cap: usize, out_n: *mut usize) -> c_int {
    status(|| {
        let f = resolve(fd)?;
        let buf = unsafe { bytes_mut(buf, cap, "buf") }?;
        let n = rustix::io::read(f.as_fd(), buf)?;
        unsafe { put(out_n, n, "out_n") }
    })
}

/// One write call. A short count is normal; the caller loops.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_write(fd: i32, buf: *const u8, len: usize, out_n: *mut usize) -> c_int {
    status(|| {
        let f = resolve(fd)?;
        let buf = unsafe { bytes(buf, len, "buf") }?;
        let n = rustix::io::write(f.as_fd(), buf)?;
        unsafe { put(out_n, n, "out_n") }
    })
}

/// Closes a shim-owned fd. Other numbers give EBADF and stay open.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_close(fd: i32) -> c_int {
    status(|| match FDS.lock().unwrap_or_else(|p| p.into_inner()).remove(&fd) {
        Some(_) => Ok(()),
        None => Err(ShimErr::errno(EBADF_RAW, "bad file descriptor (not a shim fd)")),
    })
}

// ---------------------------------------------------------------------
// Open and stat family
// ---------------------------------------------------------------------

/// open with raw OFlags and Mode bits.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_open(path: *const u8, path_len: usize, flags: i32, mode: u32, out_fd: *mut i32) -> c_int {
    status(|| {
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let fd = rustix::fs::open(&path, rustix::fs::OFlags::from_bits_retain(flags as u32), rustix::fs::Mode::from_bits_retain(mode))?;
        unsafe { put(out_fd, register(ShimFd::Owned(fd)), "out_fd") }
    })
}

/// openat. dirfd AT_FDCWD starts from the current directory.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_openat(dirfd: i32, path: *const u8, path_len: usize, flags: i32, mode: u32, out_fd: *mut i32) -> c_int {
    status(|| {
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let flags = rustix::fs::OFlags::from_bits_retain(flags as u32);
        let mode = rustix::fs::Mode::from_bits_retain(mode);
        let fd = if dirfd == AT_FDCWD {
            rustix::fs::openat(rustix::fs::CWD, &path, flags, mode)?
        } else {
            let dir = resolve(dirfd)?;
            rustix::fs::openat(dir.as_fd(), &path, flags, mode)?
        };
        unsafe { put(out_fd, register(ShimFd::Owned(fd)), "out_fd") }
    })
}

/// fstat on a shim or borrowed fd.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_fstat(fd: i32, out: *mut CStat) -> c_int {
    status(|| {
        let f = resolve(fd)?;
        let s = rustix::fs::fstat(f.as_fd())?;
        unsafe { put(out, to_cstat(&s), "out") }
    })
}

/// statat with raw AtFlags bits, for example SYMLINK_NOFOLLOW (0x100).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_statat(dirfd: i32, path: *const u8, path_len: usize, flags: i32, out: *mut CStat) -> c_int {
    status(|| {
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let flags = rustix::fs::AtFlags::from_bits_retain(flags as u32);
        let s = if dirfd == AT_FDCWD {
            rustix::fs::statat(rustix::fs::CWD, &path, flags)?
        } else {
            let dir = resolve(dirfd)?;
            rustix::fs::statat(dir.as_fd(), &path, flags)?
        };
        unsafe { put(out, to_cstat(&s), "out") }
    })
}

/// lstat of a path, filled from std metadata as fswatch/unix.rs does.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_lstat(path: *const u8, path_len: usize, out: *mut CStat) -> c_int {
    status(|| {
        use std::os::unix::fs::MetadataExt;
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let m = std::fs::symlink_metadata(&path)?;
        let s = CStat {
            dev: m.dev(),
            ino: m.ino(),
            nlink: m.nlink(),
            mode: m.mode(),
            uid: m.uid(),
            gid: m.gid(),
            rdev: m.rdev(),
            size: m.size() as i64,
            blksize: m.blksize() as i64,
            blocks: m.blocks() as i64,
            atime_sec: m.atime(),
            atime_nsec: m.atime_nsec(),
            mtime_sec: m.mtime(),
            mtime_nsec: m.mtime_nsec(),
            ctime_sec: m.ctime(),
            ctime_nsec: m.ctime_nsec(),
        };
        unsafe { put(out, s, "out") }
    })
}

/// statx, for the osvfs mod_times fast path (type plus mtime).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_statx(dirfd: i32, path: *const u8, path_len: usize, flags: i32, mask: u32, out: *mut CStatx) -> c_int {
    status(|| {
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let flags = rustix::fs::AtFlags::from_bits_retain(flags as u32);
        let want = rustix::fs::StatxFlags::from_bits_retain(mask);
        let s = if dirfd == AT_FDCWD {
            rustix::fs::statx(rustix::fs::CWD, &path, flags, want)?
        } else {
            let dir = resolve(dirfd)?;
            rustix::fs::statx(dir.as_fd(), &path, flags, want)?
        };
        let v = CStatx {
            mask: s.stx_mask,
            mode: u32::from(s.stx_mode),
            mtime_sec: s.stx_mtime.tv_sec,
            mtime_nsec: s.stx_mtime.tv_nsec,
            reserved: 0,
        };
        unsafe { put(out, v, "out") }
    })
}

/// F_GETFL as raw OFlags bits (the stdio non-blocking check).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_fcntl_getfl(fd: i32, out_flags: *mut u32) -> c_int {
    status(|| {
        let f = resolve(fd)?;
        let flags = rustix::fs::fcntl_getfl(f.as_fd())?;
        unsafe { put(out_flags, flags.bits(), "out_flags") }
    })
}

/// faccessat with raw Access and AtFlags bits (the LookPath check).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_accessat(dirfd: i32, path: *const u8, path_len: usize, access: u32, flags: i32) -> c_int {
    status(|| {
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let access = rustix::fs::Access::from_bits_retain(access);
        let flags = rustix::fs::AtFlags::from_bits_retain(flags as u32);
        if dirfd == AT_FDCWD {
            rustix::fs::accessat(rustix::fs::CWD, &path, access, flags)?;
        } else {
            let dir = resolve(dirfd)?;
            rustix::fs::accessat(dir.as_fd(), &path, access, flags)?;
        }
        Ok(())
    })
}

/// utimensat. Nsec fields take the raw UTIME_OMIT and UTIME_NOW values.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_utimensat(dirfd: i32, path: *const u8, path_len: usize, times: *const CTimestamps, flags: i32) -> c_int {
    status(|| {
        if times.is_null() {
            return Err(ShimErr::invalid("times"));
        }
        let t = unsafe { *times };
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let ts = |sec: i64, nsec: i64| rustix::fs::Timespec { tv_sec: sec, tv_nsec: nsec as _ };
        let times = rustix::fs::Timestamps { last_access: ts(t.atime_sec, t.atime_nsec), last_modification: ts(t.mtime_sec, t.mtime_nsec) };
        let flags = rustix::fs::AtFlags::from_bits_retain(flags as u32);
        if dirfd == AT_FDCWD {
            rustix::fs::utimensat(rustix::fs::CWD, &path, &times, flags)?;
        } else {
            let dir = resolve(dirfd)?;
            rustix::fs::utimensat(dir.as_fd(), &path, &times, flags)?;
        }
        Ok(())
    })
}

/// mknodat. file_type_with_mode holds the S_IFMT bits, as mknod takes.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_mknodat(dirfd: i32, path: *const u8, path_len: usize, file_type_with_mode: u32, mode: u32, dev: u64) -> c_int {
    status(|| {
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let kind = rustix::fs::FileType::from_raw_mode(file_type_with_mode);
        let mode = rustix::fs::Mode::from_bits_retain(mode);
        if dirfd == AT_FDCWD {
            rustix::fs::mknodat(rustix::fs::CWD, &path, kind, mode, dev)?;
        } else {
            let dir = resolve(dirfd)?;
            rustix::fs::mknodat(dir.as_fd(), &path, kind, mode, dev)?;
        }
        Ok(())
    })
}

/// statvfs of a path. fsid is the rustix combined value.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_statvfs(path: *const u8, path_len: usize, out: *mut CStatvfs) -> c_int {
    status(|| {
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let st = rustix::fs::statvfs(&path)?;
        let v = CStatvfs {
            bsize: st.f_bsize,
            blocks: st.f_blocks,
            bfree: st.f_bfree,
            bavail: st.f_bavail,
            files: st.f_files,
            ffree: st.f_ffree,
            fsid: st.f_fsid,
            namemax: st.f_namemax,
            frsize: st.f_frsize,
            flag: st.f_flag.bits(),
        };
        unsafe { put(out, v, "out") }
    })
}

// ---------------------------------------------------------------------
// Directory reading
// ---------------------------------------------------------------------

/// The d_type byte for a rustix FileType (from fswatch/unix.rs).
fn dirent_type(t: rustix::fs::FileType) -> u8 {
    use rustix::fs::FileType;
    match t {
        FileType::Fifo => 0x1,
        FileType::CharacterDevice => 0x2,
        FileType::Directory => 0x4,
        FileType::BlockDevice => 0x6,
        FileType::RegularFile => 0x8,
        FileType::Symlink => 0xa,
        FileType::Socket => 0xc,
        _ => 0x0,
    }
}

/// One getdents64 batch in linux_dirent64 layout, built the way
/// fswatch read_dirent builds it. A record that does not fit the
/// cap_hint batch gives EINVAL, as there. Empty output is the end.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_read_dirent(fd: i32, cap_hint: usize, out: *mut ShimBuf) -> c_int {
    status(|| {
        if cap_hint == 0 || cap_hint > (1 << 24) {
            return Err(ShimErr::invalid("cap_hint"));
        }
        let f = lookup(fd)?;
        let mut raw = vec![std::mem::MaybeUninit::<u8>::uninit(); cap_hint];
        let mut dir = rustix::fs::RawDir::new(&*f, &mut raw);
        let mut buf = vec![0u8; cap_hint];
        let mut n = 0usize;
        while let Some(ent) = dir.next() {
            let ent = ent?;
            let name = ent.file_name().to_bytes();
            let reclen = (19 + name.len() + 1 + 7) & !7;
            if n + reclen > buf.len() {
                return Err(ShimErr::errno(EINVAL_RAW, "dirent record does not fit the batch"));
            }
            let rec = &mut buf[n..n + reclen];
            rec.fill(0);
            rec[0..8].copy_from_slice(&ent.ino().to_ne_bytes());
            rec[8..16].copy_from_slice(&(ent.next_entry_cookie() as i64).to_ne_bytes());
            rec[16..18].copy_from_slice(&(reclen as u16).to_ne_bytes());
            rec[18] = dirent_type(ent.file_type());
            rec[19..19 + name.len()].copy_from_slice(name);
            n += reclen;
            if dir.is_buffer_empty() {
                break;
            }
        }
        buf.truncate(n);
        unsafe { buf_put(out, buf) }
    })
}

pub type DirentFn = extern "C" fn(env: *mut c_void, ino: u64, file_type: u32, name: *const u8, name_len: usize) -> c_int;

/// Whole-directory walk with a callback. Stops at the first non-zero
/// callback status and never calls back after it (ARCH 3.7). A panic
/// guard still wraps the walk; the callback is C, so a lust panic
/// arrives as its status return, by the boundary rule.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_readdir_each(fd: i32, cb: DirentFn, env: *mut c_void, out_calls: *mut i64, out_cb_status: *mut i32) -> c_int {
    status(|| {
        let f = lookup(fd)?;
        let mut raw = vec![std::mem::MaybeUninit::<u8>::uninit(); 8192];
        let mut dir = rustix::fs::RawDir::new(&*f, &mut raw);
        let mut calls: i64 = 0;
        let mut cb_status: i32 = 0;
        while let Some(ent) = dir.next() {
            let ent = ent?;
            let name = ent.file_name().to_bytes();
            let st = cb(env, ent.ino(), u32::from(dirent_type(ent.file_type())), name.as_ptr(), name.len());
            calls += 1;
            if st != 0 {
                cb_status = st;
                break;
            }
        }
        unsafe { put(out_calls, calls, "out_calls") }?;
        unsafe { put(out_cb_status, cb_status, "out_cb_status") }
    })
}

// ---------------------------------------------------------------------
// Poll, dup2 to stdio, terminal size
// ---------------------------------------------------------------------

/// poll over shim or borrowed fds. A negative timeout waits forever.
/// revents land back in the caller records.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_poll(fds: *mut CPollFd, count: usize, timeout_ms: i32, out_n: *mut i32) -> c_int {
    status(|| {
        if fds.is_null() && count > 0 {
            return Err(ShimErr::invalid("fds"));
        }
        let recs: &mut [CPollFd] = if count == 0 { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(fds, count) } };
        let mut owned: Vec<FdRef> = Vec::with_capacity(count);
        for r in recs.iter() {
            owned.push(resolve(r.fd)?);
        }
        let mut pollfds: Vec<rustix::event::PollFd<'_>> = owned
            .iter()
            .zip(recs.iter())
            .map(|(f, r)| rustix::event::PollFd::new(f, rustix::event::PollFlags::from_bits_retain(r.events as u16)))
            .collect();
        let ts = (timeout_ms >= 0)
            .then(|| rustix::event::Timespec { tv_sec: i64::from(timeout_ms / 1000), tv_nsec: i64::from(timeout_ms % 1000) * 1_000_000 });
        let n = rustix::event::poll(&mut pollfds, ts.as_ref())?;
        for (r, p) in recs.iter_mut().zip(pollfds.iter()) {
            r.revents = p.revents().bits() as i16;
        }
        unsafe { put(out_n, n as i32, "out_n") }
    })
}

/// dup2 onto stdout (bin/tsgo close_stdout_and_stderr).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_dup2_stdout(fd: i32) -> c_int {
    status(|| {
        let f = resolve(fd)?;
        rustix::stdio::dup2_stdout(f.as_fd())?;
        Ok(())
    })
}

/// dup2 onto stderr.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_dup2_stderr(fd: i32) -> c_int {
    status(|| {
        let f = resolve(fd)?;
        rustix::stdio::dup2_stderr(f.as_fd())?;
        Ok(())
    })
}

/// TIOCGWINSZ (the one termios call ts-rust makes).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_tcgetwinsize(fd: i32, out: *mut CWinsize) -> c_int {
    status(|| {
        let f = resolve(fd)?;
        let ws = rustix::termios::tcgetwinsize(f.as_fd())?;
        let v = CWinsize { row: ws.ws_row, col: ws.ws_col, xpixel: ws.ws_xpixel, ypixel: ws.ws_ypixel };
        unsafe { put(out, v, "out") }
    })
}

// ---------------------------------------------------------------------
// inotify, fanotify, name_to_handle_at (fswatch backends)
// ---------------------------------------------------------------------

/// inotify_init1 with raw CreateFlags bits.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_inotify_init1(flags: i32, out_fd: *mut i32) -> c_int {
    status(|| {
        let fd = rustix::fs::inotify::init(rustix::fs::inotify::CreateFlags::from_bits_retain(flags as u32))?;
        unsafe { put(out_fd, register(ShimFd::Owned(fd)), "out_fd") }
    })
}

/// inotify_add_watch. The fd must be a shim inotify fd.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_inotify_add_watch(fd: i32, path: *const u8, path_len: usize, mask: u32, out_wd: *mut i32) -> c_int {
    status(|| {
        let f = lookup(fd)?;
        let path = os_path(unsafe { bytes(path, path_len, "path") }?)?;
        let wd = rustix::fs::inotify::add_watch(&*f, &path, rustix::fs::inotify::WatchFlags::from_bits_retain(mask))?;
        unsafe { put(out_wd, wd, "out_wd") }
    })
}

/// inotify_rm_watch.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_inotify_rm_watch(fd: i32, wd: i32) -> c_int {
    status(|| {
        let f = lookup(fd)?;
        rustix::fs::inotify::remove_watch(&*f, wd)?;
        Ok(())
    })
}

/// fanotify_init through nix. Flag bits pass, as in fswatch (nix names
/// no FAN_REPORT_* bit, so from_bits_retain carries them).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_fanotify_init(flags: u32, event_f_flags: u32, out_fd: *mut i32) -> c_int {
    status(|| {
        use nix::sys::fanotify::{EventFFlags, Fanotify, InitFlags};
        let f = Fanotify::init(InitFlags::from_bits_retain(flags), EventFFlags::from_bits_retain(event_f_flags))?;
        unsafe { put(out_fd, register(ShimFd::Fanotify(f)), "out_fd") }
    })
}

/// fanotify_mark through nix. An fd that is not a fanotify group gives
/// EINVAL, as in the kernel and in fswatch.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_fanotify_mark(
    fd: i32,
    flags: u32,
    mask: u64,
    dirfd: i32,
    path: *const u8,
    path_len: usize,
    has_path: i32,
) -> c_int {
    status(|| {
        use nix::sys::fanotify::{MarkFlags, MaskFlags};
        let group = lookup(fd)?;
        let ShimFd::Fanotify(f) = &*group else {
            return Err(ShimErr::errno(EINVAL_RAW, "fd is not a fanotify group"));
        };
        let flags = MarkFlags::from_bits_retain(flags);
        let mask = MaskFlags::from_bits_retain(mask);
        let owned;
        let pathname = if has_path != 0 {
            owned = os_path(unsafe { bytes(path, path_len, "path") }?)?;
            Some(&owned)
        } else {
            None
        };
        let res = if dirfd == AT_FDCWD {
            f.mark(flags, mask, rustix::fs::CWD, pathname.map(|p| &**p))
        } else {
            let dir = resolve(dirfd)?;
            f.mark(flags, mask, dir, pathname.map(|p| &**p))
        };
        res?;
        Ok(())
    })
}

/// name_to_handle_at through the name-to-handle-at crate. A path with a
/// NUL byte gives EINVAL before the call, as fswatch does.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_name_to_handle_at(
    dirfd: i32,
    path: *const u8,
    path_len: usize,
    flags: i32,
    out_handle_type: *mut i32,
    out_mount_id: *mut i32,
    out_handle: *mut ShimBuf,
) -> c_int {
    status(|| {
        let raw = unsafe { bytes(path, path_len, "path") }?;
        if raw.contains(&0) {
            return Err(ShimErr::errno(EINVAL_RAW, "path contains a NUL byte"));
        }
        let path = os_path(raw)?;
        let (handle, mount_id) = if dirfd == AT_FDCWD {
            name_to_handle_at::name_to_handle_at(&rustix::fs::CWD, &path, flags)?
        } else {
            let dir = resolve(dirfd)?;
            name_to_handle_at::name_to_handle_at(&dir, &path, flags)?
        };
        let mount_id = match mount_id {
            name_to_handle_at::MountId::Reusable(id) => id as i32,
            name_to_handle_at::MountId::Unique(id) => id as i32,
        };
        unsafe { put(out_handle_type, handle.handle_type, "out_handle_type") }?;
        unsafe { put(out_mount_id, mount_id, "out_mount_id") }?;
        unsafe { buf_put(out_handle, handle.handle) }
    })
}
