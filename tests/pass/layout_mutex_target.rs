// Rust 1.99's SmallFutex is u8 on Windows and u32 on Linux.
// Payload alignment can hide the difference, so cover three payloads.
#![allow(dead_code)]
use std::mem::{align_of, size_of};
use std::sync::Mutex;

#[cfg(target_os = "windows")]
const UNIT_SIZE: usize = 2;
#[cfg(target_os = "windows")]
const BYTE_SIZE: usize = 3;
#[cfg(target_os = "windows")]
const WORD_SIZE: usize = 8;
#[cfg(target_os = "windows")]
const SMALL_ALIGN: usize = 1;

#[cfg(target_os = "macos")]
const UNIT_SIZE: usize = 16;
#[cfg(target_os = "macos")]
const BYTE_SIZE: usize = 16;
#[cfg(target_os = "macos")]
const WORD_SIZE: usize = 16;
#[cfg(target_os = "macos")]
const SMALL_ALIGN: usize = 8;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const UNIT_SIZE: usize = 8;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const BYTE_SIZE: usize = 8;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const WORD_SIZE: usize = 12;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const SMALL_ALIGN: usize = 4;

#[cfg(target_os = "macos")]
const WORD_ALIGN: usize = 8;
#[cfg(not(target_os = "macos"))]
const WORD_ALIGN: usize = 4;

fn shape<T>(size: usize, align: usize) {
    assert_eq!(size_of::<Mutex<T>>(), size);
    assert_eq!(align_of::<Mutex<T>>(), align);
}

fn main() {
    // These array types verify every answer during check-only cross-target
    // compilation too; the generic calls below also exercise native execution.
    let _: [u8; UNIT_SIZE] = [0; size_of::<Mutex<()>>()];
    let _: [u8; BYTE_SIZE] = [0; size_of::<Mutex<u8>>()];
    let _: [u8; WORD_SIZE] = [0; size_of::<Mutex<u32>>()];
    let _: [u8; SMALL_ALIGN] = [0; align_of::<Mutex<()>>()];
    let _: [u8; SMALL_ALIGN] = [0; align_of::<Mutex<u8>>()];
    let _: [u8; WORD_ALIGN] = [0; align_of::<Mutex<u32>>()];
    shape::<()>(UNIT_SIZE, SMALL_ALIGN);
    shape::<u8>(BYTE_SIZE, SMALL_ALIGN);
    shape::<u32>(WORD_SIZE, WORD_ALIGN);
}
