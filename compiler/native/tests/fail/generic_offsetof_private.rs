// expect: L0510
use std::mem::offset_of;
mod a {
    #[repr(C)]
    pub struct S {
        pub shown: u8,
        x: u32,
    }
}
fn off<T>(t: T) -> usize {
    let _ = &t;
    offset_of!(a::S, x)
}
fn main() {
    println!("{}", off(3));
}
