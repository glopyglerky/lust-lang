#![no_std]
#![forbid(unsafe_code)]

pub fn answer() -> i32 { *Box::new(42) }
