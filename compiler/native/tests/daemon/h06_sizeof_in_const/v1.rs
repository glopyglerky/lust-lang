struct S { a: u8, b: u8 }
const N: usize = std::mem::size_of::<S>();
fn main() { println!("{}", N); }
