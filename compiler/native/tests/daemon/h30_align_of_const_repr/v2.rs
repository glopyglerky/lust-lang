#[repr(align(32))]
struct S { a: u8 }
const A: usize = std::mem::align_of::<S>();
fn main() { println!("{}", A); }
