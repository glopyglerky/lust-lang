// expect: L0510
use std::mem::offset_of;
mod m {
    #[repr(C)]
    pub struct S {
        pub a: u8,
        b: u32,
    }
}
fn main() {
    println!("{}", offset_of!(m::S, b));
}
