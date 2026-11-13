// expect: L0110
// rustc E0308: the fn items' signatures differ.
fn a(x: u8) -> u8 { x + 1 }
fn c(x: u16) -> u8 { x as u8 }
fn main() { let w = [a, c]; println!("{}", (w[0])(1)); }
