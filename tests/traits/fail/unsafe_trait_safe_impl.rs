// rustc-error: E0200
// lust-expect: L0110
unsafe trait U {}
struct S;
impl U for S {}
fn main() { println!("x"); }
