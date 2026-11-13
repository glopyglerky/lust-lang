// expect: L0110
// rustc E0308: an array length is a usize.
fn main() { let a: [u8; true] = todo!(); println!("{}", a.len()); }
