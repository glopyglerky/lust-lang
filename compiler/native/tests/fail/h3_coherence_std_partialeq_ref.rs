// expect: L0615
// rustc E0119: core has `impl PartialEq<&B> for &A where A: PartialEq<B>`, and a
// downstream crate may add `PartialEq<Down> for Local`.
struct Local;
impl<T> PartialEq<T> for &Local { fn eq(&self, _: &T) -> bool { true } }
fn main() { println!("ok"); }
