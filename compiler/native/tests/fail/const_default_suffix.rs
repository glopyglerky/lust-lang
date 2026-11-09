// expect: L0110
// rustc E0308: the default is a u8.
struct S<const N: usize = 4u8>;
fn main() { println!("x"); }
