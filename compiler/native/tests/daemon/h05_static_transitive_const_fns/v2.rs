const fn g(x: u32) -> u32 { x + 5 }
const fn f(x: u32) -> u32 { g(x) * 2 }
static S: u32 = f(10);
fn main() { println!("{}", S); }
