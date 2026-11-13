const K: u8 = 4;
fn f(x: u8) -> u8 { match x { K | 9 => 1, _ => 0 } }
fn g(t: (u8, u8)) -> u8 { match t { (K, _) => 1, _ => 0 } }
fn main() { println!("{} {} {} {}", f(3), f(9), g((3, 0)), g((4, 0))); }
