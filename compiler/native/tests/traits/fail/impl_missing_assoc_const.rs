// rustc-error: E0046
// lust-expect: L0110
trait Lim { const MAX: u32; }
struct A;
impl Lim for A {}
fn main() { println!("x"); }
