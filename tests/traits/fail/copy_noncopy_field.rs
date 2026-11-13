// rustc-error: E0204
// lust-expect: L0110
struct S(String);
impl Clone for S { fn clone(&self) -> S { S(self.0.clone()) } }
impl Copy for S {}
fn main() { println!("x"); }
