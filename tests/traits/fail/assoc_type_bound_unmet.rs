// rustc-error: E0277
// lust-expect: L0612
trait A { type X: Copy; }
struct S;
impl A for S { type X = String; }
fn main() { println!("x"); }
