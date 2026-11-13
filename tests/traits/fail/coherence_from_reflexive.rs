// rustc-error: E0119
// lust-expect: L0615
struct S;
impl<T> From<T> for S { fn from(_t: T) -> S { S } }
fn main() { println!("ok"); }
