#![no_std]
#![forbid(unsafe_code)]

pub fn answer() -> usize { std::mem::size_of::<u16>() }
