const K: u8 = 3;
fn f(x: u8) -> u8 { if let K = x { 1 } else { 0 } }
fn main() { println!("{} {}", f(3), f(4)); }
