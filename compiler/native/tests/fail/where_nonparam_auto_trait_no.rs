// expect: L0612
// rustc E0277: `Vec<*const u8>` is not Send.
fn w<T>(_x: &T) -> u8 where Vec<T>: Send { 1 }
fn main() { println!("{}", w(&(&1u8 as *const u8))); }
