struct S { a: u64, b: u8 }
const N: usize = std::mem::size_of::<S>();
fn main() { println!("{}", N); }
