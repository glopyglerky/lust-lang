// rustc-error: E0117
// lust-expect: L0616
struct S;
impl Clone for (S, S) { fn clone(&self) -> Self { (S, S) } }
fn main() { println!("x"); }
