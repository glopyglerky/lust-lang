// rustc-error: E0199
// lust-expect: L0110
trait U {}
struct S;
unsafe impl U for S {}
fn main() { println!("x"); }
