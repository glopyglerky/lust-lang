#![no_std]
#![forbid(unsafe_code)]

pub fn answer() -> i32 { *alloc::boxed::Box::new(42) }
