// K6: Rust shim probe. Built as staticlib and as cdylib from this one source.
// Every export returns a status or a plain value; panics stay inside Rust.
#![allow(clippy::missing_safety_doc)]
use std::cell::Cell;
use std::ffi::{c_char, c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Once;

thread_local! {
    static TLS_VAL: Cell<i64> = const { Cell::new(-7) };
}

fn silence_hook_once() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|_| {}));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shim_add(a: i64, b: i64) -> i64 {
    a + b
}

/// Returns 1 if an internal panic was caught by catch_unwind, else 0.
/// Copies the panic message into out (capacity cap), NUL-terminated.
#[unsafe(no_mangle)]
pub extern "C" fn shim_catch_panic(out: *mut c_char, cap: usize) -> c_int {
    silence_hook_once();
    let r = catch_unwind(|| -> i32 { panic!("shim internal boom 42") });
    match r {
        Ok(_) => 0,
        Err(payload) => {
            if !out.is_null() && cap > 0 {
                let msg: &str = if let Some(s) = payload.downcast_ref::<&str>() {
                    s
                } else if let Some(s) = payload.downcast_ref::<String>() {
                    s
                } else {
                    "unknown payload"
                };
                let bytes = msg.as_bytes();
                let n = bytes.len().min(cap - 1);
                unsafe {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out as *mut u8, n);
                    *out.add(n) = 0;
                }
            }
            1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shim_tls_set(v: i64) {
    TLS_VAL.with(|c| c.set(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn shim_tls_get() -> i64 {
    TLS_VAL.with(|c| c.get())
}

/// Spawn n Rust threads. Thread i sets its TLS to 100+i, reads it back, and
/// returns it. Sum is written to *out_sum. Returns 0 on success.
#[unsafe(no_mangle)]
pub extern "C" fn shim_spawn_sum(n: u32, out_sum: *mut i64) -> c_int {
    let r = catch_unwind(AssertUnwindSafe(|| {
        let mut handles = Vec::new();
        for i in 0..n {
            handles.push(std::thread::spawn(move || {
                TLS_VAL.with(|c| c.set(100 + i64::from(i)));
                // A caught panic on a Rust-spawned thread must also work.
                let caught = catch_unwind(|| panic!("worker boom")).is_err();
                TLS_VAL.with(|c| c.get()) + i64::from(caught)
            }));
        }
        let mut sum: i64 = 0;
        for h in handles {
            sum += h.join().map_err(|_| -1i32)?;
        }
        Ok::<i64, i32>(sum)
    }));
    match r {
        Ok(Ok(sum)) => {
            if !out_sum.is_null() {
                unsafe { *out_sum = sum };
            }
            0
        }
        _ => 1,
    }
}

/// Like shim_spawn_sum, but the workers never panic. Each returns 100+i, so
/// the expected sum for n=4 is 406. Isolates Rust threads+TLS from unwind.
#[unsafe(no_mangle)]
pub extern "C" fn shim_spawn_sum_nopanic(n: u32, out_sum: *mut i64) -> c_int {
    let mut handles = Vec::new();
    for i in 0..n {
        handles.push(std::thread::spawn(move || {
            TLS_VAL.with(|c| c.set(100 + i64::from(i)));
            TLS_VAL.with(|c| c.get())
        }));
    }
    let mut sum: i64 = 0;
    for h in handles {
        match h.join() {
            Ok(v) => sum += v,
            Err(_) => return 1,
        }
    }
    if !out_sum.is_null() {
        unsafe { *out_sum = sum };
    }
    0
}

pub type ShimCallback = extern "C" fn(env: *mut c_void, value: i64) -> c_int;

/// Call cb(env, i) for i in 0..n. Stop at the first non-zero status and return
/// it. *out_calls receives the number of calls made.
#[unsafe(no_mangle)]
pub extern "C" fn shim_call_callback(
    cb: ShimCallback,
    env: *mut c_void,
    n: i64,
    out_calls: *mut i64,
) -> c_int {
    let mut calls: i64 = 0;
    let mut status: c_int = 0;
    for i in 0..n {
        status = cb(env, i);
        calls += 1;
        if status != 0 {
            break;
        }
    }
    if !out_calls.is_null() {
        unsafe { *out_calls = calls };
    }
    status
}

/// Allocate a shim-owned buffer of len bytes, filled with byte value fill.
/// Free only with shim_free_buffer.
#[unsafe(no_mangle)]
pub extern "C" fn shim_alloc_buffer(len: usize, fill: u8) -> *mut u8 {
    let mut v = vec![fill; len];
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Free a buffer from shim_alloc_buffer with the same len.
#[unsafe(no_mangle)]
pub extern "C" fn shim_free_buffer(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

/// Return 1 iff ptr[0..len] all equal fill. Borrows the buffer for the call.
#[unsafe(no_mangle)]
pub extern "C" fn shim_check_buffer(ptr: *const u8, len: usize, fill: u8) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    let s = unsafe { std::slice::from_raw_parts(ptr, len) };
    c_int::from(s.iter().all(|&b| b == fill))
}

/// Fill a caller-owned buffer (copy-out into caller memory). Returns len.
#[unsafe(no_mangle)]
pub extern "C" fn shim_fill_caller(ptr: *mut u8, len: usize, fill: u8) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let s = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    s.fill(fill);
    len
}
