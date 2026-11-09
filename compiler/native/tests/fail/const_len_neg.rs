// expect: L0110
// rustc E0600: `-` on the usize literal.
fn main() { let a: [u8; 4 + -1] = [1, 2, 3]; println!("{}", a.len()); }
