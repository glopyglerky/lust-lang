// Const layout queries in signatures must follow --target, including the
// early array-length evaluator that has no collected ADT field table yet.
#![allow(dead_code)]
use std::mem::{align_of, size_of};
use std::sync::{Barrier, Condvar, Mutex, OnceLock, RwLock};

#[cfg(target_os = "macos")]
const MUTEX: usize = 16;
#[cfg(target_os = "windows")]
const MUTEX: usize = 2;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const MUTEX: usize = 8;
#[cfg(target_os = "macos")]
const MUTEX_ALIGN: usize = 8;
#[cfg(target_os = "windows")]
const MUTEX_ALIGN: usize = 1;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const MUTEX_ALIGN: usize = 4;
#[cfg(target_os = "macos")]
const RWLOCK: usize = 16;
#[cfg(not(target_os = "macos"))]
const RWLOCK: usize = 12;
#[cfg(target_os = "macos")]
const ONCE: usize = 8;
#[cfg(not(target_os = "macos"))]
const ONCE: usize = 4;
#[cfg(target_os = "macos")]
const ALIGN: usize = 8;
#[cfg(not(target_os = "macos"))]
const ALIGN: usize = 4;
#[cfg(target_os = "macos")]
const CONDVAR: usize = 16;
#[cfg(not(target_os = "macos"))]
const CONDVAR: usize = 4;
#[cfg(target_os = "macos")]
const BARRIER: usize = 48;
#[cfg(not(target_os = "macos"))]
const BARRIER: usize = 40;

struct Buffer {
    bytes: [u8; size_of::<Condvar>()],
}

fn mutex_bytes() -> [u8; size_of::<Mutex<()>>()] { [0; MUTEX] }
fn mutex_alignment_bytes() -> [u8; align_of::<Mutex<()>>()] { [0; MUTEX_ALIGN] }
fn rwlock_bytes() -> [u8; size_of::<RwLock<()>>()] { [0; RWLOCK] }
fn once_bytes() -> [u8; size_of::<OnceLock<()>>()] { [0; ONCE] }
fn alignment_bytes() -> [u8; align_of::<RwLock<()>>()] { [0; ALIGN] }
fn barrier_bytes() -> [u8; size_of::<Barrier>()] { [0; BARRIER] }

fn main() {
    let buffer = Buffer { bytes: [0; CONDVAR] };
    println!("{} {} {} {} {} {} {}", mutex_bytes().len(), mutex_alignment_bytes().len(), rwlock_bytes().len(),
        once_bytes().len(), alignment_bytes().len(), buffer.bytes.len(), barrier_bytes().len());
}
