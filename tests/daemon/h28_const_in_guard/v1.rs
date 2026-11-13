const K: u8 = 3;
fn f(x: u8) -> u8 { match x { v if v == K => 1, _ => 0 } }
fn main() { println!("{} {}", f(3), f(4)); }
