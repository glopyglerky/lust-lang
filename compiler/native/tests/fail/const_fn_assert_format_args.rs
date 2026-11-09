// expect: L0631
// rustc E0015: a formatting message in a const fn.
const fn chk(x: u8) -> u8 { assert!(x < 9, "bad {}", "x"); x }
const C: u8 = chk(8);
fn main() { println!("{}", C); }
