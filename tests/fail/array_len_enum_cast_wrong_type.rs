// expect: L0110
// rustc E0308: an array length is a usize, not a u8.
enum C { A, B, Count }
fn main() { let x = [0u8; C::Count as u8]; let _ = (C::A, C::B); println!("{}", x.len()); }
