// rustc-error: E0326
// lust-expect: L0110
trait T { const K: u32; }
struct X;
impl T for X { const K: u8 = 3; }
fn main() { println!("x"); }
