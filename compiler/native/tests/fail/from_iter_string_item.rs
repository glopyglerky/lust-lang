// expect: L0612
// rustc E0277: a `String` cannot be built from `u32` items.
fn main() { let s: String = String::from_iter([1u32, 2]); println!("{}", s); }
