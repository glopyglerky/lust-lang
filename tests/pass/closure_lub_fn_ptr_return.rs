// A closure match arm set joins as one fn pointer return.
fn pick(n: u8) -> fn(u8) -> u8 { match n { 0 => |x| x, 1 => |x| x + 1, _ => |x| x * 2 } }
fn main() { println!("{} {} {}", pick(0)(5), pick(1)(5), pick(2)(5)); }
