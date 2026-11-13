#![no_std]
#![forbid(unsafe_code)]

mod other { pub const Vec: u8 = 1; }

pub fn value() -> Vec<u8> { Vec::new() }
