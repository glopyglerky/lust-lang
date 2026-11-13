// expect: L0110
// rustc E0605: a cast of an enum with fields is not a primitive cast.
#[allow(dead_code)]
enum C { A(u8), B, Count }
fn main() { let x = [0u8; C::Count as usize]; println!("{}", x.len()); }
