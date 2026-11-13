// rustc-error: E0322
// lust-expect: L0110
struct S;
impl Sized for S {}
fn main() { println!("x"); }
