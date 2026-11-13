// Host sync state affects nested layouts and RawVec's first allocation.
#![allow(dead_code)]
use std::mem::{align_of, size_of};
use std::sync::{Condvar, Mutex, OnceLock, RwLock};

enum Empty {}
#[repr(align(16))]
struct Aligned(u8);
struct Pair<T> {
    before: u8,
    value: T,
    after: u16,
}

fn shape<T>() {
    println!("{} {} {} {} {} {} {}",
        size_of::<T>(), align_of::<T>(),
        size_of::<Option<T>>(), size_of::<Option<Option<T>>>(),
        size_of::<Result<T, u8>>(), size_of::<Pair<T>>(),
        size_of::<[T; 3]>());
}

fn main() {
    shape::<Mutex<()>>();
    shape::<Mutex<bool>>();
    shape::<Mutex<Aligned>>();
    shape::<Mutex<Empty>>();
    shape::<RwLock<()>>();
    shape::<RwLock<bool>>();
    shape::<RwLock<Aligned>>();
    shape::<RwLock<Empty>>();
    shape::<OnceLock<()>>();
    shape::<OnceLock<bool>>();
    shape::<OnceLock<Aligned>>();
    shape::<OnceLock<Empty>>();
    shape::<Condvar>();
    shape::<std::sync::Barrier>();

    // These payloads put the complete host type on opposite sides of
    // RawVec's 1024-byte initial-capacity boundary on Linux and macOS.
    let locks: Vec<Mutex<[u8; 1016]>> = (0..1).filter(|_| true)
        .map(|_| Mutex::new([0; 1016])).collect();
    let cells: Vec<OnceLock<[u8; 1020]>> = (0..1).filter(|_| true)
        .map(|_| OnceLock::new()).collect();
    println!("{} {} {} {}", locks.len(), locks.capacity(), cells.len(), cells.capacity());
}
