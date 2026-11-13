const LO: u8 = 2;
const HI: u8 = 9;
fn f(x: u8) -> u8 { match x { LO..=HI => 1, _ => 0 } }
fn main() { println!("{} {} {}", f(1), f(3), f(9)); }
