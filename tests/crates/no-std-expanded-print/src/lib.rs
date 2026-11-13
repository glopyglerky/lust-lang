#![no_std]
#![forbid(unsafe_code)]

macro_rules! emit { () => { println!("hello") }; }
pub fn answer() -> i32 { emit!(); 42 }
