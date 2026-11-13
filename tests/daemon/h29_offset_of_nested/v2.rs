#[repr(C)]
struct Inner { a: u32, b: u8 }
#[repr(C)]
struct Outer { x: u8, inner: Inner }
fn main() { println!("{}", std::mem::offset_of!(Outer, inner, b)); }
