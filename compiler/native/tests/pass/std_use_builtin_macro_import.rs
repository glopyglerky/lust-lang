// Imports of std macros lust expands by name bind no alias, and a
// `std::primitive` import is not a missing path: all stay accepted.
use std::mem::offset_of;
#[allow(unused_imports)]
use std::primitive::u8 as Byte;
use std::dbg;

#[allow(dead_code)]
#[repr(C)]
struct S {
    a: u8,
    b: u32,
}

fn main() {
    let x: u8 = 3;
    let _ = dbg!(x);
    println!("{} {}", offset_of!(S, b), x);
}
