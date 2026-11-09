// expect: L0104
// rustc E0277: `Box<u8>: From<&str>` does not hold.
fn main() { let b: Box<u8> = "x".into(); println!("{}", b); }
