const fn k() -> u8 { 3 }
const K: u8 = k();
fn f(x: u8) -> u8 { match x { K => 1, _ => 0 } }
fn main() { println!("{} {}", f(3), f(4)); }
