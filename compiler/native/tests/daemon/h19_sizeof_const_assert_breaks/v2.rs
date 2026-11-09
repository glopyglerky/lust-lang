struct S { a: u64, b: u64 }
const _: () = assert!(std::mem::size_of::<S>() == 8);
fn main() { println!("ok"); }
