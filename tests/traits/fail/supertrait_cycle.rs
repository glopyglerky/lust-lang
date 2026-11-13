// rustc-error: E0391
// lust-expect: L0110
trait A: B {}
trait B: A {}
fn main() { println!("x"); }
