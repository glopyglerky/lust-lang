// expect: L0110
// rustc E0308: a range of char and u8.
fn main() { println!("{}", ('a'..=b'z').contains(&'q')); }
