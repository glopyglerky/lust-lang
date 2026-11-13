#![no_std]
#![forbid(unsafe_code)]

mod child { pub fn answer() -> usize { Vec::<u8>::new().len() } }
pub fn answer() -> usize { child::answer() }
