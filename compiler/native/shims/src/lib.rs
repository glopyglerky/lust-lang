//! liblust_shim: the rustc-built cdylib for ts-rust system crates.
//!
//! ARCH 3.7 and README.md in this directory state the contract. The
//! short form: every export catches its own panics, only scalars, byte
//! buffers, C records, raw fd numbers, and u64 handles cross, and shim
//! memory is freed only by the shim.
//!
//! Modules:
//! - this file: status codes, the panic guard, last-error text, owned
//!   buffers, the allocator bridge, and shared helpers.
//! - `fd`: files, pipes, poll, termios, inotify, fanotify, handles.
//! - `proc`: processes, limits, prctl, signal masks, signal-hook.
//! - `data`: gzip, xxh3, jiff time zones, BigInt whole-value calls.
//! - `jem`: jemalloc controls (feature `jemalloc`) or UNSUPPORTED.

// The exports are C ABI functions: pointer validity is the contract
// in lust_shim.h, as in C. Every pointer is null-checked and every
// buffer takes an explicit length before a dereference, and the
// panic guard turns a bad call that panics into a status, so the
// lint that wants each such function marked `unsafe` in Rust terms
// does not model this boundary.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, Once};

mod data;
mod fd;
mod jem;
mod proc;

/// ABI version. Must match LUST_SHIM_ABI_VERSION in the header.
pub const ABI_VERSION: u32 = 1;

pub const OK: i32 = 0;
pub const ERR_PANIC: i32 = -1;
pub const ERR_INVALID: i32 = -2;
pub const ERR: i32 = -3;
pub const ERR_UNSUPPORTED: i32 = -4;

/// One failure: a status code and a one-line message for last_error.
pub struct ShimErr {
    pub status: i32,
    pub msg: String,
}

impl ShimErr {
    pub fn invalid(what: &str) -> ShimErr {
        ShimErr { status: ERR_INVALID, msg: format!("invalid argument: {what}") }
    }
    pub fn msg(text: impl std::fmt::Display) -> ShimErr {
        ShimErr { status: ERR, msg: text.to_string() }
    }
    pub fn unsupported(what: &str) -> ShimErr {
        ShimErr { status: ERR_UNSUPPORTED, msg: format!("unsupported: {what}") }
    }
    /// An errno failure. The status is the raw errno, as Go errors are.
    pub fn errno(raw: i32, text: impl std::fmt::Display) -> ShimErr {
        ShimErr { status: raw, msg: text.to_string() }
    }
}

impl From<rustix::io::Errno> for ShimErr {
    fn from(e: rustix::io::Errno) -> ShimErr {
        ShimErr::errno(e.raw_os_error(), e)
    }
}

impl From<nix::errno::Errno> for ShimErr {
    fn from(e: nix::errno::Errno) -> ShimErr {
        ShimErr::errno(e as i32, e)
    }
}

impl From<std::io::Error> for ShimErr {
    fn from(e: std::io::Error) -> ShimErr {
        match e.raw_os_error() {
            Some(raw) => ShimErr::errno(raw, e),
            None => ShimErr::msg(e),
        }
    }
}

pub type ShimResult<T> = Result<T, ShimErr>;

thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Records the message for lust_shim_last_error on this thread.
pub fn set_last_error(msg: &str) {
    LAST_ERROR.with(|e| {
        let mut e = e.borrow_mut();
        e.clear();
        e.push_str(msg);
    });
}

/// Silences the default Rust panic output once. The cdylib owns every
/// Rust panic in a lust dev process, and the message already reaches
/// the caller through last_error (the K6 probe used this same shape).
fn silence_panic_hook_once() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|_| {}));
    });
}

/// Runs one export body. A panic becomes ERR_PANIC with its message.
/// An Err becomes its own status. Success clears no state.
pub fn guard<T>(f: impl FnOnce() -> ShimResult<T>) -> Result<T, i32> {
    silence_panic_hook_once();
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => {
            set_last_error(&e.msg);
            Err(e.status)
        }
        Err(payload) => {
            let msg: &str = if let Some(s) = payload.downcast_ref::<&str>() {
                s
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s
            } else {
                "unknown panic payload"
            };
            set_last_error(msg);
            Err(ERR_PANIC)
        }
    }
}

/// guard for exports whose C result is only the status code.
pub fn status(f: impl FnOnce() -> ShimResult<()>) -> c_int {
    match guard(f) {
        Ok(()) => OK,
        Err(s) => s,
    }
}

/// Borrows a byte buffer argument. A null pointer is valid only with a
/// zero length, which gives an empty slice.
///
/// # Safety
/// `ptr` must be readable for `len` bytes, or null with `len` zero,
/// for the length of the call (the C contract in lust_shim.h).
pub unsafe fn bytes<'a>(ptr: *const u8, len: usize, what: &str) -> ShimResult<&'a [u8]> {
    if ptr.is_null() {
        if len == 0 {
            return Ok(&[]);
        }
        return Err(ShimErr::invalid(what));
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

/// Borrows a mutable byte buffer argument. Null rules as in `bytes`.
///
/// # Safety
/// `ptr` must be writable for `len` bytes, or null with `len` zero,
/// for the length of the call.
pub unsafe fn bytes_mut<'a>(ptr: *mut u8, len: usize, what: &str) -> ShimResult<&'a mut [u8]> {
    if ptr.is_null() {
        if len == 0 {
            return Ok(&mut []);
        }
        return Err(ShimErr::invalid(what));
    }
    Ok(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
}

/// Writes one output value. A null out pointer is an argument error.
///
/// # Safety
/// A non-null `out` must be writable for one `T`.
pub unsafe fn put<T>(out: *mut T, value: T, what: &str) -> ShimResult<()> {
    if out.is_null() {
        return Err(ShimErr::invalid(what));
    }
    unsafe { *out = value };
    Ok(())
}

/// A path argument as an OS path. Paths are raw bytes (Go strings in
/// ts-rust hold OS bytes). An inner NUL byte is EINVAL, as in Go.
pub fn os_path(raw: &[u8]) -> ShimResult<std::path::PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    if raw.contains(&0) {
        return Err(ShimErr::errno(22, "path contains a NUL byte"));
    }
    Ok(std::ffi::OsStr::from_bytes(raw).to_os_string().into())
}

/// The C shape of a shim-owned buffer. Must match LustShimBuf.
#[repr(C)]
pub struct ShimBuf {
    pub ptr: *mut u8,
    pub len: usize,
    pub cap: usize,
}

/// Moves a Vec into a caller LustShimBuf. The struct must be zeroed or
/// hold a live shim buffer (which is freed first). The caller frees
/// the result with lust_shim_buf_free and with nothing else.
///
/// # Safety
/// A non-null `out` must point to a LustShimBuf that is zeroed or
/// holds a live shim buffer. Nothing else may be freed through it.
pub unsafe fn buf_put(out: *mut ShimBuf, mut v: Vec<u8>) -> ShimResult<()> {
    if out.is_null() {
        return Err(ShimErr::invalid("out buf"));
    }
    let slot = unsafe { &mut *out };
    if !slot.ptr.is_null() {
        let old = unsafe { Vec::from_raw_parts(slot.ptr, slot.len, slot.cap) };
        drop(old);
    }
    slot.ptr = v.as_mut_ptr();
    slot.len = v.len();
    slot.cap = v.capacity();
    std::mem::forget(v);
    Ok(())
}

/// A handle table for one state type. Handles start at 1. One table
/// per type keeps a bad handle of the wrong type an argument error.
pub struct Handles<T> {
    map: LazyLock<Mutex<HashMap<u64, T>>>,
    next: AtomicU64,
}

impl<T> Default for Handles<T> {
    fn default() -> Handles<T> {
        Handles::new()
    }
}

impl<T> Handles<T> {
    pub const fn new() -> Handles<T> {
        Handles { map: LazyLock::new(|| Mutex::new(HashMap::new())), next: AtomicU64::new(1) }
    }
    pub fn insert(&self, value: T) -> u64 {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        self.map.lock().unwrap_or_else(|p| p.into_inner()).insert(id, value);
        id
    }
    /// Runs f on the value under the table lock. f must be quick and
    /// must not call back into the same table.
    pub fn with<R>(&self, id: u64, what: &str, f: impl FnOnce(&mut T) -> ShimResult<R>) -> ShimResult<R> {
        let mut map = self.map.lock().unwrap_or_else(|p| p.into_inner());
        let value = map.get_mut(&id).ok_or_else(|| ShimErr::invalid(what))?;
        f(value)
    }
    pub fn remove(&self, id: u64, what: &str) -> ShimResult<T> {
        self.map.lock().unwrap_or_else(|p| p.into_inner()).remove(&id).ok_or_else(|| ShimErr::invalid(what))
    }
}

// ---------------------------------------------------------------------
// Core exports
// ---------------------------------------------------------------------

/// The ABI version of this cdylib. Never fails.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_abi_version() -> u32 {
    ABI_VERSION
}

/// Copies this thread's last error. Returns its full byte length.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_last_error(out: *mut c_char, cap: usize) -> usize {
    LAST_ERROR.with(|e| {
        let e = e.borrow();
        let bytes = e.as_bytes();
        if !out.is_null() && cap > 0 {
            let n = bytes.len().min(cap - 1);
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), out as *mut u8, n);
                *out.add(n) = 0;
            }
        }
        bytes.len()
    })
}

/// Frees a shim-owned buffer and zeroes the struct. A null struct
/// pointer or a zeroed struct does nothing.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_buf_free(buf: *mut ShimBuf) {
    if buf.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let slot = unsafe { &mut *buf };
        if !slot.ptr.is_null() {
            let v = unsafe { Vec::from_raw_parts(slot.ptr, slot.len, slot.cap) };
            drop(v);
        }
        slot.ptr = std::ptr::null_mut();
        slot.len = 0;
        slot.cap = 0;
    }));
}

/// Test support. The panic must come back as a status (K6 case).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_panic_for_test() -> c_int {
    status(|| -> ShimResult<()> { panic!("lust shim forced panic 42") })
}

// ---------------------------------------------------------------------
// Allocator bridge (ARCH 3.7). When lust selects the jemalloc shim
// build, its runtime alloc intrinsic calls these exports and so every
// lust allocation lands in jemalloc arenas, under thread.arena. In a
// plain build they route to the system allocator.
// ---------------------------------------------------------------------

/// A Layout from C size and align. Bad aligns are argument errors.
fn layout(size: usize, align: usize) -> ShimResult<std::alloc::Layout> {
    std::alloc::Layout::from_size_align(size, align).map_err(|_| ShimErr::invalid("size or align"))
}

/// Allocates size bytes at align. Out of memory gives a null pointer
/// with status OK, so the runtime can run its own OOM path.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_alloc(size: usize, align: usize, out_ptr: *mut *mut u8) -> c_int {
    status(|| {
        let p = unsafe { std::alloc::alloc(layout(size, align)?) };
        unsafe { put(out_ptr, p, "out_ptr") }
    })
}

/// Zeroed form of lust_shim_alloc.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_alloc_zeroed(size: usize, align: usize, out_ptr: *mut *mut u8) -> c_int {
    status(|| {
        let p = unsafe { std::alloc::alloc_zeroed(layout(size, align)?) };
        unsafe { put(out_ptr, p, "out_ptr") }
    })
}

/// Grows or shrinks a block from lust_shim_alloc. A null result keeps
/// the old block valid, as std::alloc::realloc does.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_realloc(ptr: *mut u8, old_size: usize, align: usize, new_size: usize, out_ptr: *mut *mut u8) -> c_int {
    status(|| {
        if ptr.is_null() {
            return Err(ShimErr::invalid("ptr"));
        }
        let p = unsafe { std::alloc::realloc(ptr, layout(old_size, align)?, new_size) };
        unsafe { put(out_ptr, p, "out_ptr") }
    })
}

/// Frees a block from the alloc bridge. A null pointer does nothing.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_dealloc(ptr: *mut u8, size: usize, align: usize) {
    if ptr.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Ok(l) = layout(size, align) {
            unsafe { std::alloc::dealloc(ptr, l) };
        }
    }));
}
